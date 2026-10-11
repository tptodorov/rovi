#!/usr/bin/env bash
# Laptop camera frames named by wall-clock time in µs since the epoch, so they line up with client timestamps
# (python `time.time()`) and the host-side serial log. Also writes OUTDIR/sheet.jpg, a contact sheet.
# Usage: scripts/cam.sh OUTDIR [SECONDS=5] [DEVICE=/dev/video0]    (needs ffmpeg; see shell.nix)
set -euo pipefail
out=${1:?usage: cam.sh OUTDIR [SECONDS] [DEVICE]}; secs=${2:-5}; dev=${3:-/dev/video0}
mkdir -p "$out/frames"
# MJPEG is copied, not re-encoded, so every frame is a JPEG as the camera sent it.
ffmpeg -loglevel error -y -f v4l2 -input_format mjpeg -video_size 1280x720 -framerate 30 \
  -use_wallclock_as_timestamps 1 -t "$secs" -i "$dev" -c:v copy -copyts -f image2 -frame_pts 1 "$out/frames/%d.jpg"
step=$(( $(ls "$out/frames" | wc -l) / 12 + 1 ))  # a sheet of 12 frames spread over the whole recording
ffmpeg -loglevel error -y -pattern_type glob -i "$out/frames/*.jpg" \
  -vf "select='not(mod(n,$step))',scale=320:-1,tile=4x3" -frames:v 1 "$out/sheet.jpg"
