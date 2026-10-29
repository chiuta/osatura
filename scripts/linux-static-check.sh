#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
# Checks that a Linux build of `osatura` is fully static and works with no system libraries at all:
# no ELF interpreter, no DT_NEEDED entries, and demo/check/render/export run inside an empty chroot
# (no libc, no /etc, no /usr). Needs readelf and sudo (for chroot). Usage: scripts/linux-static-check.sh path/to/osatura
set -euo pipefail
bin=${1:?usage: $0 path/to/osatura}
if readelf -lW "$bin" | grep -q 'Requesting program interpreter'; then echo "not static: has an ELF interpreter" >&2; exit 1; fi
if readelf -dW "$bin" | grep -q '(NEEDED)'; then echo "not static: needs shared libraries:" >&2; readelf -dW "$bin" | grep NEEDED >&2; exit 1; fi
readelf -hW "$bin" | grep -E 'Type:'
readelf -lW "$bin" | grep -qE 'GNU_RELRO' && echo "RELRO: yes"
root=$(mktemp -d)
trap 'sudo rm -rf "$root"' EXIT
cp "$bin" "$root/osatura"
run() { sudo chroot "$root" /osatura "$@"; }
run --version | head -1
run demo-person /p.json
run check /p.json
run render /p.json 0 /p.png
for f in gif apng lottie glb html json; do run export /p.json "/out.$f" >/dev/null 2>&1; test -s "$root/out.$f"; echo "export .$f ok"; done
echo "static build OK: runs with no system libraries"
