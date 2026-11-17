// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//
// OSATURA for Unity: P/Invoke bindings for the C ABI (osatura.h, version 1) and a MonoBehaviour that plays a project
// onto a SpriteRenderer / RawImage texture. Put the native library (osatura_capi.dll, libosatura_capi.so or
// libosatura_capi.dylib, built from the osatura-capi crate) in Assets/Plugins/<platform>/ and the project JSON in a
// TextAsset. NOT run inside Unity by the author: the declared function names are checked against osatura.h by
// scripts/check-adapters.py; everything else is untested here. Report problems upstream.
using System;
using System.Runtime.InteropServices;
using System.Text;
using UnityEngine;

namespace Osatura
{
    /// <summary>Raw bindings, one per C function used. Names and argument types follow osatura.h.</summary>
    public static class Native
    {
        const string Lib = "osatura_capi";
        [DllImport(Lib)] public static extern uint osatura_abi_version();
        [DllImport(Lib)] public static extern IntPtr osatura_version();
        [DllImport(Lib)] public static extern IntPtr osatura_open(byte[] json, UIntPtr len);
        [DllImport(Lib)] public static extern void osatura_close(IntPtr engine);
        [DllImport(Lib)] public static extern IntPtr osatura_last_error();
        [DllImport(Lib)] public static extern uint osatura_frame_count(IntPtr engine);
        [DllImport(Lib)] public static extern float osatura_fps(IntPtr engine);
        [DllImport(Lib)] public static extern uint osatura_width(IntPtr engine);
        [DllImport(Lib)] public static extern uint osatura_height(IntPtr engine);
        [DllImport(Lib)] public static extern int osatura_render_rgba(IntPtr engine, float frame, float scale, byte[] outBuf, UIntPtr cap, out uint w, out uint h);
        [DllImport(Lib)] public static extern int osatura_render_rgba(IntPtr engine, float frame, float scale, IntPtr outBuf, UIntPtr cap, out uint w, out uint h);
        [DllImport(Lib)] public static extern uint osatura_machine_count(IntPtr engine);
        [DllImport(Lib)] public static extern IntPtr osatura_player_open(IntPtr engine, byte[] machineUtf8Z);
        [DllImport(Lib)] public static extern void osatura_player_close(IntPtr player);
        [DllImport(Lib)] public static extern int osatura_player_set_bool(IntPtr player, byte[] nameUtf8Z, int value);
        [DllImport(Lib)] public static extern int osatura_player_set_number(IntPtr player, byte[] nameUtf8Z, double value);
        [DllImport(Lib)] public static extern int osatura_player_fire(IntPtr player, byte[] triggerUtf8Z);
        [DllImport(Lib)] public static extern int osatura_player_advance(IntPtr player, double dt);
        [DllImport(Lib)] public static extern int osatura_player_render_rgba(IntPtr player, float scale, byte[] outBuf, UIntPtr cap, out uint w, out uint h);

        public static byte[] Z(string s) { var b = Encoding.UTF8.GetBytes(s); var z = new byte[b.Length + 1]; Array.Copy(b, z, b.Length); return z; }
        public static string Str(IntPtr p) => p == IntPtr.Zero ? "" : Marshal.PtrToStringUTF8(p);
    }

    /// <summary>An opened project. Dispose it (or let the component do it) to free the native engine.</summary>
    public sealed class Project : IDisposable
    {
        public const uint ExpectedAbi = 1;
        IntPtr h;
        public int Frames => (int)Native.osatura_frame_count(h);
        public float Fps => Native.osatura_fps(h);
        public int Width => (int)Native.osatura_width(h);
        public int Height => (int)Native.osatura_height(h);
        public IntPtr Handle => h;

        public Project(string json)
        {
            if (Native.osatura_abi_version() != ExpectedAbi)
                throw new InvalidOperationException("OSATURA native library has ABI " + Native.osatura_abi_version() + ", this adapter expects " + ExpectedAbi);
            var b = Encoding.UTF8.GetBytes(json);
            h = Native.osatura_open(b, (UIntPtr)b.Length);
            if (h == IntPtr.Zero) throw new ArgumentException("OSATURA could not open the project: " + Native.Str(Native.osatura_last_error()));
        }

        /// <summary>Render a frame (fractional frames are allowed) into <paramref name="tex"/>, resizing it when needed.
        /// Rows are flipped because Unity textures start at the bottom.</summary>
        public void RenderTo(ref Texture2D tex, float frame, float scale = 1f)
        {
            uint w, hh;
            int r = Native.osatura_render_rgba(h, frame, scale, (byte[])null, UIntPtr.Zero, out w, out hh);
            if (r != -2 && r != 0) throw new InvalidOperationException("render failed (" + r + "): " + Native.Str(Native.osatura_last_error()));
            var buf = new byte[checked((int)(w * hh * 4))];
            r = Native.osatura_render_rgba(h, frame, scale, buf, (UIntPtr)buf.Length, out w, out hh);
            if (r != 0) throw new InvalidOperationException("render failed (" + r + "): " + Native.Str(Native.osatura_last_error()));
            int stride = (int)w * 4;
            var flipped = new byte[buf.Length];
            for (int y = 0; y < hh; y++) Buffer.BlockCopy(buf, y * stride, flipped, (int)(hh - 1 - y) * stride, stride);
            if (tex == null || tex.width != (int)w || tex.height != (int)hh)
            {
                if (tex != null) UnityEngine.Object.Destroy(tex);
                tex = new Texture2D((int)w, (int)hh, TextureFormat.RGBA32, false) { wrapMode = TextureWrapMode.Clamp };
            }
            tex.LoadRawTextureData(flipped);
            tex.Apply(false);
        }

        public void Dispose() { if (h != IntPtr.Zero) { Native.osatura_close(h); h = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~Project() { if (h != IntPtr.Zero) Native.osatura_close(h); }
    }

    /// <summary>Plays an OSATURA project on a SpriteRenderer (or any renderer's main texture). Rendering happens on the CPU
    /// every frame the animation advances, so keep <see cref="scale"/> modest; for many instances, export a sprite sheet instead.</summary>
    [AddComponentMenu("OSATURA/Osatura Player")]
    public class OsaturaPlayer : MonoBehaviour
    {
        public TextAsset project;
        [Range(0.05f, 4f)] public float scale = 1f;
        public bool loop = true;
        public float speed = 1f;
        public float pixelsPerUnit = 100f;
        Project p; Texture2D tex; SpriteRenderer sr; float t; Sprite sprite;

        void OnEnable()
        {
            if (project == null) { enabled = false; Debug.LogError("OsaturaPlayer: no project assigned", this); return; }
            try { p = new Project(project.text); } catch (Exception e) { enabled = false; Debug.LogError(e.Message, this); return; }
            sr = GetComponent<SpriteRenderer>(); if (sr == null) sr = gameObject.AddComponent<SpriteRenderer>();
            t = 0; Draw(0);
        }
        void OnDisable() { p?.Dispose(); p = null; if (tex != null) Destroy(tex); if (sprite != null) Destroy(sprite); }
        void Update()
        {
            if (p == null || p.Frames <= 0) return;
            t += Time.deltaTime * speed * p.Fps;
            float last = p.Frames - 1;
            if (loop && p.Frames > 1) t = Mathf.Repeat(t, p.Frames); else t = Mathf.Clamp(t, 0, last);
            Draw(t);
        }
        void Draw(float frame)
        {
            var old = tex; p.RenderTo(ref tex, Mathf.Min(frame, Mathf.Max(0, p.Frames - 1)), scale);
            if (sprite == null || old != tex) { if (sprite != null) Destroy(sprite); sprite = Sprite.Create(tex, new Rect(0, 0, tex.width, tex.height), new Vector2(0.5f, 0.5f), pixelsPerUnit); sr.sprite = sprite; }
        }
    }
}
