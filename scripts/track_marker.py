#!/usr/bin/env python3
"""Tracks one ArUco marker (DICT_4X4_50) through the frames scripts/cam.sh writes (`frames/<epoch µs>.jpg`) into a CSV.
Needs OpenCV (shell.nix). Frames without the marker, or with it cut off, give no row.

    python3 scripts/track_marker.py FRAMES_DIR out.csv [--id 0] [--side-m 0.06] [--calib calib.json]

Columns: epoch_us (from the file name), x_px and y_px (marker centre, image axes: y down), heading_rad (direction of the
marker's top edge, 0 = image right, counter-clockwise as seen in the image), size_px (mean side length). With --side-m
also x_m, y_m: pixels times one scale (side over the median size_px), y_m pointing up the image. That scale is only
right when the marker moves in a plane parallel to the image, i.e. a camera looking straight down. --calib (from
scripts/calibrate_camera.py) undistorts the corners first.
"""
import argparse, csv, json, math, pathlib, statistics, sys
from typing import NamedTuple

import cv2
import numpy as np

DICT = cv2.aruco.DICT_4X4_50


class Row(NamedTuple):
    epoch_us: int
    x_px: float
    y_px: float
    heading_rad: float
    size_px: float


class Detection(NamedTuple):
    x_px: float
    y_px: float
    heading_rad: float
    size_px: float


_detector = cv2.aruco.ArucoDetector(cv2.aruco.getPredefinedDictionary(DICT), cv2.aruco.DetectorParameters())


def detect(img, marker_id=0, calib=None):
    """The marker's Detection in a BGR or grey image, or None. ArUco only reports markers with all four corners in view."""
    corners, ids, _ = _detector.detectMarkers(img)
    if ids is None or marker_id not in ids.ravel():
        return None
    c = corners[list(ids.ravel()).index(marker_id)].reshape(4, 2).astype(np.float64)  # TL, TR, BR, BL as printed
    if calib:
        k, dist = np.array(calib["camera_matrix"], np.float64), np.array(calib["dist_coeffs"], np.float64)
        c = cv2.undistortPoints(c.reshape(-1, 1, 2), k, dist, P=k).reshape(4, 2)
    x, y = c.mean(axis=0)
    top = c[1] - c[0]
    size = np.mean([np.linalg.norm(c[i] - c[i - 1]) for i in range(4)])
    return Detection(float(x), float(y), math.atan2(-top[1], top[0]), float(size))


def frame_time_us(path):
    """The capture time from a cam.sh frame name (`<epoch µs>.jpg`), or None for any other file."""
    return int(path.stem) if path.suffix == ".jpg" and path.stem.isdigit() else None


def track(frames_dir, marker_id=0, calib=None):
    """Rows for the frames that show the marker, in capture order. Unreadable frames are skipped."""
    rows = []
    for t, p in sorted((frame_time_us(p), p) for p in pathlib.Path(frames_dir).iterdir() if frame_time_us(p) is not None):
        img = cv2.imread(str(p))
        d = detect(img, marker_id, calib) if img is not None else None
        if d:
            rows.append(Row(t, *d))
    return rows


def write_csv(rows, out, side_m=None):
    fields = list(Row._fields) + (["x_m", "y_m"] if side_m else [])
    scale = side_m / statistics.median(r.size_px for r in rows) if side_m and rows else None
    with open(out, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(fields)
        for r in rows:
            w.writerow([r.epoch_us, f"{r.x_px:.2f}", f"{r.y_px:.2f}", f"{r.heading_rad:.5f}", f"{r.size_px:.2f}"]
                       + ([f"{r.x_px * scale:.5f}", f"{-r.y_px * scale:.5f}"] if scale else []))


def main(argv=None):
    a = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    a.add_argument("frames", help="the frames/ directory written by scripts/cam.sh")
    a.add_argument("out", help="CSV to write")
    a.add_argument("--id", type=int, default=0, help="marker id (default 0)")
    a.add_argument("--side-m", type=float, help="printed marker side in metres (black square only); adds x_m, y_m")
    a.add_argument("--calib", help="camera calibration JSON from scripts/calibrate_camera.py")
    args = a.parse_args(argv)
    calib = json.loads(pathlib.Path(args.calib).read_text()) if args.calib else None
    rows = track(args.frames, args.id, calib)
    write_csv(rows, args.out, args.side_m)
    total = sum(frame_time_us(p) is not None for p in pathlib.Path(args.frames).iterdir())
    print(f"marker {args.id} in {len(rows)} of {total} frames -> {args.out}")
    if not rows:
        sys.exit(1)


if __name__ == "__main__":
    main()
