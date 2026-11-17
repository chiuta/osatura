# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
@tool
extends EditorPlugin

func _enter_tree() -> void:
	add_custom_type("OsaturaSheet", "Sprite2D", preload("osatura_sheet.gd"), null)

func _exit_tree() -> void:
	remove_custom_type("OsaturaSheet")
