#!/usr/bin/env bash
# usage: build_flash.sh <name> [ENV=VAL...]  (stops background monitor first)
set -euo pipefail
name=$1; shift
cd /home/todor/.paseo/worktrees/08pcg0yd/feat-m3-wifi-queue/experiments/rovi-m3-wifi-queue
nix-shell ../../shell.nix --run ". ~/export-esp.sh && export ROVI_WIFI_PASSWORD=\"\$(cat ~/.config/rovi/m3-wifi-pass)\" && env $* CARGO_TARGET_DIR=/tmp/rovi-m3/$name cargo build --release --locked 2>&1 | tail -1 && espflash flash --non-interactive --port /dev/ttyACM0 /tmp/rovi-m3/$name/xtensa-esp32s3-none-elf/release/rovi-m3-wifi-queue 2>&1 | tail -1"
