// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! wgpu back end: offscreen supersampled RGBA8 canvas, stencil-then-cover fills and strokes, WGSL gradients,
//! layer compositing with opacity / masks / blend modes, box-filter resolve and read-back.

use crate::tess::{self, StrokeSpec};
use osatura_core::engine::{Engine, Frame, NodeKind, RenderNode};
use osatura_core::math::Affine2;
use osatura_core::path::width_outline;
use osatura_core::style::{Blend, Paint};
use std::time::Instant;
use wgpu::util::DeviceExt;

const COLOR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const STENCIL: wgpu::TextureFormat = wgpu::TextureFormat::Stencil8;
const MAX_STOPS: usize = 16;
const DRAW_BYTES: usize = 32 + 32 + MAX_STOPS * 32; // header(32) + inv/geom(32)... see `encode_draw`
const MITER_LIMIT: f32 = 4.0; // same as the CPU rasteriser (tiny-skia default)

#[derive(Debug)]
pub enum GpuError {
    /// No usable adapter (message says what was tried).
    NoAdapter(String),
    /// The scene uses something this renderer does not implement.
    Unsupported(String),
    /// wgpu reported an error.
    Device(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::NoAdapter(s) => write!(f, "no GPU adapter: {s}"),
            GpuError::Unsupported(s) => write!(f, "not supported by the GPU renderer: {s}"),
            GpuError::Device(s) => write!(f, "GPU error: {s}"),
        }
    }
}
impl std::error::Error for GpuError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterChoice {
    /// Whatever wgpu prefers (a real GPU when there is one).
    Auto,
    /// A CPU implementation (Vulkan lavapipe / llvmpipe); falls back to wgpu's own fallback adapter.
    Software,
}

impl std::str::FromStr for AdapterChoice {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "auto" => Ok(AdapterChoice::Auto),
            "software" => Ok(AdapterChoice::Software),
            _ => Err(format!("unknown adapter '{s}' (use auto or software)")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AdapterSummary {
    pub name: String,
    pub backend: String,
    pub device_type: String,
    /// True for CPU implementations (llvmpipe, lavapipe, SwiftShader ...). Timings on those say nothing about GPUs.
    pub is_software: bool,
}

impl std::fmt::Display for AdapterSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{} / {}]", self.name, self.backend, self.device_type)
    }
}

#[derive(Clone, Debug)]
pub struct GpuOptions {
    /// Output pixels per stage unit (same meaning as `RasterOptions::scale`).
    pub scale: f32,
    /// Supersampling factor per axis (anti-aliasing by 4x4 box filter by default). Reduced automatically if the
    /// canvas would exceed the adapter's texture size limit.
    pub supersample: u32,
    /// Skip the project background.
    pub transparent: bool,
    /// The CPU rasteriser's bone overlay is not implemented on the GPU; asking for it is an error.
    pub show_bones: bool,
}

impl Default for GpuOptions {
    fn default() -> Self {
        GpuOptions { scale: 1.0, supersample: 4, transparent: false, show_bones: false }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Timing {
    /// Tessellation + command recording on the CPU.
    pub record_ms: f64,
    /// Submit -> results mapped (includes execution and read-back).
    pub gpu_ms: f64,
    pub supersample: u32,
    pub passes: usize,
    pub layers: usize,
}

/// Premultiplied RGBA8 result.
pub struct GpuImage {
    pub width: u32,
    pub height: u32,
    pub premul_rgba: Vec<u8>,
    pub timing: Timing,
}

impl GpuImage {
    pub fn to_pixmap(&self) -> Option<tiny_skia::Pixmap> {
        let size = tiny_skia::IntSize::from_wh(self.width, self.height)?;
        tiny_skia::Pixmap::from_vec(self.premul_rgba.clone(), size)
    }
    pub fn encode_png(&self) -> Result<Vec<u8>, String> {
        self.to_pixmap().ok_or("bad image size")?.encode_png().map_err(|e| e.to_string())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    NonZero = 0,
    EvenOdd = 1,
    Any = 2,
}

struct Pipes {
    bgl_draw: wgpu::BindGroupLayout,
    bgl_comp: wgpu::BindGroupLayout,
    bgl_down: wgpu::BindGroupLayout,
    stencil: [wgpu::RenderPipeline; 3],
    cover: wgpu::RenderPipeline,
    comp: wgpu::RenderPipeline,
    down: wgpu::RenderPipeline,
}

pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub adapter: AdapterSummary,
    pipes: Pipes,
    dummy: wgpu::TextureView,
    uniform_align: usize,
}

fn summarize(info: &wgpu::AdapterInfo) -> AdapterSummary {
    let is_software = info.device_type == wgpu::DeviceType::Cpu
        || ["llvmpipe", "lavapipe", "swiftshader", "software"].iter().any(|k| info.name.to_ascii_lowercase().contains(k));
    AdapterSummary {
        name: info.name.clone(),
        backend: format!("{:?}", info.backend),
        device_type: format!("{:?}", info.device_type),
        is_software,
    }
}

impl GpuContext {
    /// Open an adapter. `OSATURA_GPU_BACKEND=vulkan|gl` restricts the back end (for testing both paths).
    pub fn new(choice: AdapterChoice) -> Result<GpuContext, GpuError> {
        let backends = match std::env::var("OSATURA_GPU_BACKEND").ok().as_deref() {
            Some("vulkan") => wgpu::Backends::VULKAN,
            Some("gl") => wgpu::Backends::GL,
            _ => wgpu::Backends::PRIMARY | wgpu::Backends::GL,
        };
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = backends;
        let instance = wgpu::Instance::new(desc);
        let adapter = match choice {
            AdapterChoice::Auto => pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            }))
            .map_err(|e| GpuError::NoAdapter(e.to_string()))?,
            AdapterChoice::Software => {
                let all = pollster::block_on(instance.enumerate_adapters(backends));
                let cpu: Vec<_> = all.into_iter().filter(|a| summarize(&a.get_info()).is_software).collect();
                // Prefer Vulkan (lavapipe) over GL (llvmpipe) when both exist.
                let pick = cpu.iter().position(|a| a.get_info().backend == wgpu::Backend::Vulkan).or(if cpu.is_empty() { None } else { Some(0) });
                match pick {
                    Some(i) => cpu.into_iter().nth(i).unwrap(),
                    None => pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                        force_fallback_adapter: true,
                        ..Default::default()
                    }))
                    .map_err(|e| GpuError::NoAdapter(format!("no software adapter (install mesa-vulkan-drivers for lavapipe): {e}")))?,
                }
            }
        };
        let summary = summarize(&adapter.get_info());
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("osatura-gpu"),
            ..Default::default()
        }))
        .map_err(|e| GpuError::Device(e.to_string()))?;
        let uniform_align = device.limits().min_uniform_buffer_offset_alignment.max(256) as usize;
        let pipes = build_pipes(&device);
        let dummy = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("dummy"),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: COLOR,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok(GpuContext { device, queue, adapter: summary, pipes, dummy, uniform_align })
    }

    pub fn max_texture_dim(&self) -> u32 {
        self.device.limits().max_texture_dimension_2d
    }
}

fn build_pipes(device: &wgpu::Device) -> Pipes {
    let draw_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("draw"),
        source: wgpu::ShaderSource::Wgsl(include_str!("wgsl/draw.wgsl").into()),
    });
    let comp_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("composite"),
        source: wgpu::ShaderSource::Wgsl(include_str!("wgsl/composite.wgsl").into()),
    });
    let down_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("downsample"),
        source: wgpu::ShaderSource::Wgsl(include_str!("wgsl/downsample.wgsl").into()),
    });
    let ubo = |binding: u32, dynamic: bool, size: usize, vis: wgpu::ShaderStages| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: vis,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: dynamic,
            min_binding_size: wgpu::BufferSize::new(size as u64),
        },
        count: None,
    };
    let tex = |binding: u32| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let both = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let bgl_draw = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bgl_draw"),
        entries: &[ubo(0, true, DRAW_BYTES, both)],
    });
    let bgl_comp = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bgl_comp"),
        entries: &[ubo(0, false, 32, wgpu::ShaderStages::FRAGMENT), tex(1), tex(2), tex(3)],
    });
    let bgl_down = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bgl_down"),
        entries: &[ubo(0, false, 16, wgpu::ShaderStages::FRAGMENT), tex(1)],
    });
    let layout = |bgl: &wgpu::BindGroupLayout| {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(bgl)],
            immediate_size: 0,
        })
    };
    let vbuf = wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 }],
    };
    let face = |compare, pass_op| wgpu::StencilFaceState {
        compare,
        fail_op: wgpu::StencilOperation::Keep,
        depth_fail_op: wgpu::StencilOperation::Keep,
        pass_op,
    };
    let ds = |front: wgpu::StencilFaceState, back: wgpu::StencilFaceState| wgpu::DepthStencilState {
        format: STENCIL,
        depth_write_enabled: Some(false),
        depth_compare: Some(wgpu::CompareFunction::Always),
        stencil: wgpu::StencilState { front, back, read_mask: 0xff, write_mask: 0xff },
        bias: wgpu::DepthBiasState::default(),
    };
    let draw_layout = layout(&bgl_draw);
    let make_draw = |label: &str, frag: &str, ds: wgpu::DepthStencilState, write: wgpu::ColorWrites, blend: Option<wgpu::BlendState>| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&draw_layout),
            vertex: wgpu::VertexState {
                module: &draw_mod,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(vbuf.clone())],
            },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: Some(ds),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &draw_mod,
                entry_point: Some(frag),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: COLOR, blend, write_mask: write })],
            }),
            multiview_mask: None,
            cache: None,
        })
    };
    use wgpu::{CompareFunction as C, StencilOperation as Op};
    let nz = ds(face(C::Always, Op::IncrementWrap), face(C::Always, Op::DecrementWrap));
    let eo = ds(face(C::Always, Op::Invert), face(C::Always, Op::Invert));
    let any = ds(face(C::Always, Op::Replace), face(C::Always, Op::Replace));
    let cover_ds = ds(face(C::NotEqual, Op::Zero), face(C::NotEqual, Op::Zero));
    let none = wgpu::ColorWrites::empty();
    let stencil = [
        make_draw("stencil_nz", "fs_stencil", nz, none, None),
        make_draw("stencil_eo", "fs_stencil", eo, none, None),
        make_draw("stencil_any", "fs_stencil", any, none, None),
    ];
    let premul_over = wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
    };
    let cover = make_draw("cover", "fs_cover", cover_ds, wgpu::ColorWrites::ALL, Some(premul_over));
    let full = |label: &str, module: &wgpu::ShaderModule, frag: &str, bgl: &wgpu::BindGroupLayout| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&layout(bgl)),
            vertex: wgpu::VertexState { module, entry_point: Some("vs_full"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some(frag),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: COLOR, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview_mask: None,
            cache: None,
        })
    };
    let comp = full("composite", &comp_mod, "fs_comp", &bgl_comp);
    let down = full("downsample", &down_mod, "fs_down", &bgl_down);
    Pipes { bgl_draw, bgl_comp, bgl_down, stencil, cover, comp, down }
}

// ---------------------------------------------------------------------------------------------
// Uniform encoding
// ---------------------------------------------------------------------------------------------

fn fin(v: f32) -> f32 {
    if v.is_finite() { v } else { 0.0 }
}

fn push_f32(out: &mut Vec<u8>, vals: &[f32]) {
    for v in vals {
        out.extend_from_slice(&fin(*v).to_ne_bytes());
    }
}

fn push_u32(out: &mut Vec<u8>, vals: &[u32]) {
    for v in vals {
        out.extend_from_slice(&v.to_ne_bytes());
    }
}

/// Layout must match `struct Draw` in draw.wgsl: size(8) kind(4) nstops(4) | inv_abcd(16) | inv_t(16) | geom(16) | stops_c | stops_o.
fn encode_draw(target: (f32, f32), paint: &Paint, inv: &Affine2) -> Result<Vec<u8>, GpuError> {
    let col = |c: osatura_core::style::Rgba| [fin(c.r).clamp(0.0, 1.0), fin(c.g).clamp(0.0, 1.0), fin(c.b).clamp(0.0, 1.0), fin(c.a).clamp(0.0, 1.0)];
    let mut stops_c = [[0.0f32; 4]; MAX_STOPS];
    let mut stops_o = [[0.0f32; 4]; MAX_STOPS];
    let (mut kind, mut nstops) = (0u32, 1u32);
    let mut geom = [0.0f32; 4];
    let mut load = |stops: &[osatura_core::style::Stop], base: osatura_core::style::Rgba| -> Result<u32, GpuError> {
        if stops.len() > MAX_STOPS {
            return Err(GpuError::Unsupported(format!("gradient with {} stops (limit {MAX_STOPS})", stops.len())));
        }
        if stops.is_empty() {
            stops_c[0] = col(base);
            return Ok(0);
        }
        let mut s = stops.to_vec();
        s.sort_by(|a, b| a.offset.total_cmp(&b.offset));
        for (i, st) in s.iter().enumerate() {
            stops_c[i] = col(st.color);
            stops_o[i][0] = fin(st.offset).clamp(0.0, 1.0);
        }
        Ok(s.len() as u32)
    };
    match paint {
        _ if paint.degenerate_color().is_some() => stops_c[0] = col(paint.degenerate_color().unwrap()),
        Paint::Solid(c) => stops_c[0] = col(*c),
        Paint::Linear { from, to, stops } => {
            // Degenerate axis or no stops: the CPU rasteriser falls back to the first stop colour.
            if from.distance(*to) < 1e-6 || stops.is_empty() {
                stops_c[0] = col(paint.base_color());
            } else {
                nstops = load(stops, paint.base_color())?;
                kind = 1;
                geom = [from.x, from.y, to.x, to.y];
            }
        }
        Paint::Radial { center, radius, stops } => {
            if stops.is_empty() {
                stops_c[0] = col(paint.base_color());
            } else {
                nstops = load(stops, paint.base_color())?;
                kind = 2;
                geom = [center.x, center.y, radius.max(0.001), 0.0];
            }
        }
    }
    let mut out = Vec::with_capacity(DRAW_BYTES);
    push_f32(&mut out, &[target.0, target.1]);
    push_u32(&mut out, &[kind, nstops]);
    push_f32(&mut out, &[inv.a, inv.b, inv.c, inv.d, inv.tx, inv.ty, 0.0, 0.0]);
    push_f32(&mut out, &geom);
    for c in &stops_c {
        push_f32(&mut out, c);
    }
    for o in &stops_o {
        push_f32(&mut out, o);
    }
    debug_assert_eq!(out.len(), DRAW_BYTES);
    Ok(out)
}

fn blend_index(b: Blend) -> u32 {
    match b {
        Blend::Normal => 0,
        Blend::Multiply => 1,
        Blend::Screen => 2,
        Blend::Overlay => 3,
        Blend::Darken => 4,
        Blend::Lighten => 5,
        Blend::ColorDodge => 6,
        Blend::ColorBurn => 7,
        Blend::HardLight => 8,
        Blend::SoftLight => 9,
        Blend::Difference => 10,
        Blend::Exclusion => 11,
        Blend::Add => 12,
    }
}

// ---------------------------------------------------------------------------------------------
// Recording
// ---------------------------------------------------------------------------------------------

struct Layer {
    tex: wgpu::Texture,
    view: wgpu::TextureView,
}

struct Job {
    mode: Mode,
    v0: u32,
    vn: u32,
    c0: u32,
    uoff: u32,
}

struct Rec<'a> {
    ctx: &'a GpuContext,
    frame: &'a Frame,
    enc: wgpu::CommandEncoder,
    w: u32,
    h: u32,
    ss: u32,
    scale: f32,
    pool: Vec<Layer>,
    stencil: wgpu::TextureView,
    passes: usize,
    layers: usize,
}

impl Rec<'_> {
    fn new_layer(&mut self) -> Layer {
        self.layers += 1;
        let tex = self.ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("layer"),
            size: wgpu::Extent3d { width: self.w, height: self.h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: COLOR,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        Layer { tex, view }
    }

    fn clear(&mut self, l: &Layer, c: wgpu::Color) {
        self.passes += 1;
        let _ = self.enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &l.view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(c), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }

    fn acquire(&mut self) -> Layer {
        let l = self.pool.pop().unwrap_or_else(|| self.new_layer());
        self.clear(&l, wgpu::Color::TRANSPARENT);
        l
    }

    fn release(&mut self, l: Layer) {
        self.pool.push(l);
    }

    fn draw_node(&mut self, target: &Layer, id: usize, force_visible: bool, simple: bool) -> Result<(), GpuError> {
        let frame = self.frame;
        let node = &frame.nodes[id];
        if !node.visible && !force_visible {
            return Ok(());
        }
        let need_layer = node.opacity < 0.999 || (!simple && (node.blend != Blend::Normal || node.mask.is_some()));
        if !need_layer {
            return self.draw_content(target, id, force_visible, simple);
        }
        let tmp = self.acquire();
        self.draw_content(&tmp, id, force_visible, simple)?;
        let mask = match (simple, node.mask) {
            (false, Some((mid, invert))) => {
                let mp = self.acquire();
                self.draw_node(&mp, mid, true, true)?;
                Some((mp, invert))
            }
            _ => None,
        };
        let mode = if simple { 0 } else { blend_index(node.blend) };
        self.composite(target, &tmp, mask.as_ref().map(|(m, i)| (m, *i)), node.opacity.clamp(0.0, 1.0), mode);
        self.release(tmp);
        if let Some((mp, _)) = mask {
            self.release(mp);
        }
        Ok(())
    }

    fn draw_content(&mut self, target: &Layer, id: usize, force_visible: bool, simple: bool) -> Result<(), GpuError> {
        let frame = self.frame;
        let node = &frame.nodes[id];
        match node.kind {
            NodeKind::Vector => self.draw_paths(target, node),
            NodeKind::Group | NodeKind::Switch => {
                for &c in &node.children {
                    self.draw_node(target, c, force_visible, simple)?;
                }
                Ok(())
            }
            NodeKind::Image => Err(GpuError::Unsupported(format!("image layer '{}' (bitmaps and image meshes are CPU-only)", node.name))),
        }
    }

    fn composite(&mut self, target: &Layer, src: &Layer, mask: Option<(&Layer, bool)>, opacity: f32, mode: u32) {
        let scratch = self.pool.pop().unwrap_or_else(|| self.new_layer());
        let size = wgpu::Extent3d { width: self.w, height: self.h, depth_or_array_layers: 1 };
        self.enc.copy_texture_to_texture(target.tex.as_image_copy(), scratch.tex.as_image_copy(), size);
        let mut u = Vec::new();
        push_u32(&mut u, &[mode, mask.is_some() as u32, mask.map_or(0, |m| m.1 as u32), 0]);
        push_f32(&mut u, &[opacity, 0.0, 0.0, 0.0]);
        let ubuf = self.ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("comp_u"),
            contents: &u,
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let mask_view = mask.map_or(&self.ctx.dummy, |m| &m.0.view);
        let bg = self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("comp_bg"),
            layout: &self.ctx.pipes.bgl_comp,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&scratch.view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&src.view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(mask_view) },
            ],
        });
        self.passes += 1;
        {
            let mut pass = self.enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("composite"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.ctx.pipes.comp);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
        self.release(scratch);
    }

    fn draw_paths(&mut self, target: &Layer, node: &RenderNode) -> Result<(), GpuError> {
        let k = self.scale * self.ss as f32;
        let m = Affine2::scale(k).mul(node.world);
        let Some(inv) = m.inverse() else { return Ok(()) };
        if !(m.a.is_finite() && m.b.is_finite() && m.c.is_finite() && m.d.is_finite() && m.tx.is_finite() && m.ty.is_finite()) {
            return Ok(());
        }
        let tol = 0.1 / m.a.hypot(m.b).max(m.c.hypot(m.d)).max(0.01);
        let (tw, th) = (self.w as f32, self.h as f32);
        let mut verts: Vec<f32> = Vec::new();
        let mut uni: Vec<u8> = Vec::new();
        let mut jobs: Vec<Job> = Vec::new();
        let align = self.ctx.uniform_align;
        let mut add = |tris: tess::Tris, mode: Mode, paint: &Paint| -> Result<(), GpuError> {
            if tris.is_empty() {
                return Ok(());
            }
            let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for p in &tris {
                if !(p[0].is_finite() && p[1].is_finite()) {
                    return Ok(());
                }
                x0 = x0.min(p[0]);
                y0 = y0.min(p[1]);
                x1 = x1.max(p[0]);
                y1 = y1.max(p[1]);
            }
            let (x0, y0, x1, y1) = (x0.max(0.0) - 1.0, y0.max(0.0) - 1.0, x1.min(tw) + 1.0, y1.min(th) + 1.0);
            if x1 <= x0 || y1 <= y0 {
                return Ok(());
            }
            let data = encode_draw((tw, th), paint, &inv)?;
            let uoff = uni.len();
            uni.extend_from_slice(&data);
            uni.resize(uoff + align.max(DRAW_BYTES.div_ceil(align) * align), 0);
            let v0 = (verts.len() / 2) as u32;
            for p in &tris {
                verts.extend_from_slice(p);
            }
            let c0 = (verts.len() / 2) as u32;
            verts.extend_from_slice(&[x0, y0, x1, y0, x1, y1, x0, y0, x1, y1, x0, y1]);
            jobs.push(Job { mode, v0, vn: tris.len() as u32, c0, uoff: uoff as u32 });
            Ok(())
        };
        for path in &node.paths {
            if path.anchors.len() < 2 {
                continue;
            }
            if let Some(fill) = &path.fill {
                let poly = tess::map_poly(&path.flatten(tol), &m);
                add(tess::fill_fan(&poly), Mode::NonZero, fill)?;
            }
            if let Some(st) = &path.stroke {
                if st.width_profile.is_empty() {
                    if st.width <= 0.0 {
                        continue; // zero width: the CPU rasteriser would draw a hairline, which is not implemented here
                    }
                    let spec = StrokeSpec { width: st.width, cap: st.cap, join: st.join, miter_limit: MITER_LIMIT };
                    let pts = path.flatten(tol);
                    let mut tris = tess::Tris::new();
                    if tess::dash_valid(&st.dash) {
                        for piece in tess::dash(&pts, path.closed, &st.dash, st.dash_offset).map_err(GpuError::Unsupported)? {
                            tris.extend(tess::stroke_tris(&piece, false, &spec, &m));
                        }
                    } else {
                        tris = tess::stroke_tris(&pts, path.closed, &spec, &m);
                    }
                    add(tris, Mode::Any, &st.paint)?;
                } else if let Some(out) = width_outline(&path.polyline(tol), st) {
                    let mut tris = tess::Tris::new();
                    let mode = if out.even_odd { Mode::EvenOdd } else { Mode::NonZero };
                    for poly in &out.polys {
                        tris.extend(tess::fill_fan(&tess::map_poly(poly, &m)));
                    }
                    // Both rings of a closed outline share one stencil pass (even-odd), open outlines are one polygon.
                    add(tris, mode, &st.paint)?;
                }
            }
        }
        if jobs.is_empty() {
            return Ok(());
        }
        let dev = &self.ctx.device;
        let vb = dev.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("verts"),
            contents: &verts.iter().flat_map(|v| v.to_ne_bytes()).collect::<Vec<u8>>(),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ub = dev.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("draw_u"),
            contents: &uni,
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bg = dev.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("draw_bg"),
            layout: &self.ctx.pipes.bgl_draw,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &ub, offset: 0, size: wgpu::BufferSize::new(DRAW_BYTES as u64) }),
            }],
        });
        self.passes += 1;
        let mut pass = self.enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("paths"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.stencil,
                depth_ops: None,
                stencil_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0), store: wgpu::StoreOp::Discard }),
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_vertex_buffer(0, vb.slice(..));
        for j in &jobs {
            pass.set_bind_group(0, &bg, &[j.uoff]);
            pass.set_pipeline(&self.ctx.pipes.stencil[j.mode as usize]);
            pass.set_stencil_reference(if j.mode == Mode::Any { 1 } else { 0 });
            pass.draw(j.v0..j.v0 + j.vn, 0..1);
            pass.set_pipeline(&self.ctx.pipes.cover);
            pass.set_stencil_reference(0);
            pass.draw(j.c0..j.c0 + 6, 0..1);
        }
        Ok(())
    }
}

/// Render one already evaluated frame.
pub fn render_frame(ctx: &GpuContext, engine: &Engine, f: &Frame, opts: &GpuOptions) -> Result<GpuImage, GpuError> {
    let guard = ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let r = render_inner(ctx, engine, f, opts);
    let err = pollster::block_on(guard.pop());
    match (r, err) {
        (Ok(_), Some(e)) | (Err(GpuError::Device(_)), Some(e)) => Err(GpuError::Device(e.to_string())),
        (r, _) => r,
    }
}

fn render_inner(ctx: &GpuContext, engine: &Engine, f: &Frame, opts: &GpuOptions) -> Result<GpuImage, GpuError> {
    if opts.show_bones {
        return Err(GpuError::Unsupported("bone overlay (--bones) is CPU-only".into()));
    }
    if !(opts.scale.is_finite() && opts.scale > 0.0) {
        return Err(GpuError::Unsupported("scale must be positive".into()));
    }
    let p = &engine.project;
    let w = ((p.width * opts.scale).round() as u32).max(1);
    let h = ((p.height * opts.scale).round() as u32).max(1);
    let maxd = ctx.max_texture_dim();
    if w.max(h) > maxd {
        return Err(GpuError::Unsupported(format!("output {w}x{h} exceeds the adapter texture limit {maxd}")));
    }
    let ss = opts.supersample.clamp(1, 8).min(maxd / w.max(h)).max(1);
    let (sw, sh) = (w * ss, h * ss);
    let t0 = Instant::now();
    let dev = &ctx.device;
    let stencil = dev
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("stencil"),
            size: wgpu::Extent3d { width: sw, height: sh, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: STENCIL,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut rec = Rec {
        ctx,
        frame: f,
        enc: dev.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("osatura") }),
        w: sw,
        h: sh,
        ss,
        scale: opts.scale,
        pool: Vec::new(),
        stencil,
        passes: 0,
        layers: 0,
    };
    let canvas = rec.new_layer();
    let bg = match (p.background, opts.transparent) {
        (Some(c), false) => {
            let cl = |v: f32| fin(v).clamp(0.0, 1.0) as f64;
            let a = cl(c.a);
            wgpu::Color { r: cl(c.r) * a, g: cl(c.g) * a, b: cl(c.b) * a, a }
        }
        _ => wgpu::Color::TRANSPARENT,
    };
    rec.clear(&canvas, bg);
    for &root in &f.roots {
        rec.draw_node(&canvas, root, false, false)?;
    }
    // Resolve (box filter) into the final texture and read it back.
    let out_tex = dev.create_texture(&wgpu::TextureDescriptor {
        label: Some("out"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: COLOR,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let out_view = out_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let mut du = Vec::new();
    push_u32(&mut du, &[ss, 0, 0, 0]);
    let dbuf = dev.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("down_u"), contents: &du, usage: wgpu::BufferUsages::UNIFORM });
    let dbg = dev.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("down_bg"),
        layout: &ctx.pipes.bgl_down,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: dbuf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&canvas.view) },
        ],
    });
    {
        let mut pass = rec.enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("resolve"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &out_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&ctx.pipes.down);
        pass.set_bind_group(0, &dbg, &[]);
        pass.draw(0..3, 0..1);
    }
    let row = (w as usize * 4).div_ceil(256) * 256;
    let rb = dev.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (row * h as usize) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    rec.enc.copy_texture_to_buffer(
        out_tex.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &rb,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row as u32), rows_per_image: Some(h) },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    let (passes, layers) = (rec.passes, rec.layers);
    let cmd = rec.enc.finish();
    let record_ms = t0.elapsed().as_secs_f64() * 1e3;
    let t1 = Instant::now();
    ctx.queue.submit([cmd]);
    let (tx, rx) = std::sync::mpsc::channel();
    rb.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    dev.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| GpuError::Device(e.to_string()))?;
    rx.recv().map_err(|e| GpuError::Device(e.to_string()))?.map_err(|e| GpuError::Device(e.to_string()))?;
    let gpu_ms = t1.elapsed().as_secs_f64() * 1e3;
    let mut data = Vec::with_capacity(w as usize * h as usize * 4);
    {
        let view = rb.slice(..).get_mapped_range().map_err(|e| GpuError::Device(e.to_string()))?;
        for y in 0..h as usize {
            data.extend_from_slice(&view[y * row..y * row + w as usize * 4]);
        }
    }
    rb.unmap();
    // Keep the premultiplied invariant (rounding can leave colour 1 LSB above alpha).
    for px in data.chunks_exact_mut(4) {
        let a = px[3];
        for c in &mut px[..3] {
            *c = (*c).min(a);
        }
    }
    Ok(GpuImage { width: w, height: h, premul_rgba: data, timing: Timing { record_ms, gpu_ms, supersample: ss, passes, layers } })
}

/// Evaluate `frame` and render it.
pub fn render(ctx: &GpuContext, engine: &Engine, frame: f32, opts: &GpuOptions) -> Result<GpuImage, GpuError> {
    let f = engine.evaluate(frame);
    render_frame(ctx, engine, &f, opts)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_choice_parses_and_rejects() {
        assert_eq!("auto".parse::<AdapterChoice>(), Ok(AdapterChoice::Auto));
        assert_eq!("software".parse::<AdapterChoice>(), Ok(AdapterChoice::Software));
        assert!("cuda".parse::<AdapterChoice>().unwrap_err().contains("unknown adapter"));
    }

    #[test]
    fn blend_indices_match_the_wgsl_table() {
        use Blend::*;
        let all = [Normal, Multiply, Screen, Overlay, Darken, Lighten, ColorDodge, ColorBurn, HardLight, SoftLight, Difference, Exclusion, Add];
        for (i, b) in all.iter().enumerate() {
            assert_eq!(blend_index(*b), i as u32);
        }
    }

    #[test]
    fn draw_uniform_has_the_declared_size_and_limits_are_enforced() {
        let inv = Affine2::IDENTITY;
        let solid = encode_draw((10.0, 10.0), &Paint::solid("#ff0000"), &inv).unwrap();
        assert_eq!(solid.len(), DRAW_BYTES);
        let many: Vec<_> = (0..17).map(|i| osatura_core::style::Stop { offset: i as f32 / 16.0, color: osatura_core::style::Rgba::BLACK }).collect();
        let g = Paint::Linear { from: osatura_core::math::Vec2::new(0.0, 0.0), to: osatura_core::math::Vec2::new(1.0, 0.0), stops: many };
        assert!(matches!(encode_draw((1.0, 1.0), &g, &inv), Err(GpuError::Unsupported(_))));
        // A degenerate axis falls back to the first stop colour (kind 0), like the CPU rasteriser.
        let flat = Paint::Linear {
            from: osatura_core::math::Vec2::new(2.0, 2.0),
            to: osatura_core::math::Vec2::new(2.0, 2.0),
            stops: vec![osatura_core::style::Stop { offset: 0.0, color: osatura_core::style::Rgba::WHITE }],
        };
        let b = encode_draw((1.0, 1.0), &flat, &inv).unwrap();
        assert_eq!(u32::from_ne_bytes(b[8..12].try_into().unwrap()), 0);
    }
}
