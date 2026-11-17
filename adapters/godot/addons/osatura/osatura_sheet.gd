# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
#
# OSATURA for Godot 4: plays a sprite sheet exported with
#     osatura export project.json hero.png --columns 8 --scale 0.5
# (the PNG plus hero.atlas.json). Pure GDScript, no native code, so it works on every platform Godot exports to.
# NOT run inside Godot by the author: scripts/check-adapters.py checks that every atlas key read here exists in the real
# CLI output; the rest is untested here. Report problems upstream.
@tool
class_name OsaturaSheet
extends Sprite2D

## The sprite sheet PNG written by `osatura export … .png`.
@export var sheet: Texture2D:
	set(v):
		sheet = v
		_rebuild()
## The atlas JSON next to it (`hero.atlas.json`).
@export_file("*.json") var atlas_path: String:
	set(v):
		atlas_path = v
		_rebuild()
@export var playing: bool = true
@export var loop: bool = true
@export var speed: float = 1.0

signal finished

var _frames: Array = []
var _fps: float = 24.0
var _t: float = 0.0
var _idx: int = -1

func _ready() -> void:
	_rebuild()

func _rebuild() -> void:
	_frames = []
	if sheet == null or atlas_path == "" or not FileAccess.file_exists(atlas_path):
		return
	var parsed = JSON.parse_string(FileAccess.get_file_as_string(atlas_path))
	if typeof(parsed) != TYPE_DICTIONARY:
		push_error("OsaturaSheet: %s is not an OSATURA atlas" % atlas_path)
		return
	_fps = float(parsed.get("fps", 24.0))
	_frames = parsed.get("frames", [])
	texture = sheet
	region_enabled = true
	_t = 0.0
	_idx = -1
	_show(0)

func frame_count() -> int:
	return _frames.size()

## Jump to a frame (clamped).
func seek(frame: int) -> void:
	_t = float(clampi(frame, 0, maxi(0, _frames.size() - 1)))
	_show(int(_t))

func _show(i: int) -> void:
	if _frames.is_empty() or i == _idx:
		return
	_idx = i
	var f: Dictionary = _frames[i]
	region_rect = Rect2(float(f["x"]), float(f["y"]), float(f["w"]), float(f["h"]))

func _process(delta: float) -> void:
	if not playing or _frames.size() < 2:
		return
	_t += delta * speed * _fps
	var n := _frames.size()
	if _t >= n:
		if loop:
			_t = fmod(_t, float(n))
		else:
			_t = float(n - 1)
			playing = false
			finished.emit()
	_show(int(_t))
