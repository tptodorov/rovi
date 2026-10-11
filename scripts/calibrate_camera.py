#!/usr/bin/env python3
"""Camera calibration from checkerboard photos into the JSON scripts/track_marker.py --calib reads. Needs OpenCV.

    python3 scripts/calibrate_camera.py calib.json shots/*.jpg [--cols 9] [--rows 6] [--square-m 0.025]

--cols and --rows count inner corners (a board of 10x7 squares has 9x6). Take 10-20 shots of the flat, printed board
with the same camera and resolution as the tracking run, tilted and in different parts of the image.
"""
import argparse, json, sys

import cv2
import numpy as np


def calibrate(paths, cols, rows, square_m):
    """{camera_matrix, dist_coeffs, image_size, rms_px, images_used}; exits if fewer than 3 boards are found."""
    board = np.zeros((rows * cols, 3), np.float32)
    board[:, :2] = np.mgrid[0:cols, 0:rows].T.reshape(-1, 2) * square_m
    obj, img, size = [], [], None
    for p in paths:
        grey = cv2.imread(str(p), cv2.IMREAD_GRAYSCALE)
        if grey is None:
            continue
        found, corners = cv2.findChessboardCorners(grey, (cols, rows))
        if not found:
            continue
        corners = cv2.cornerSubPix(grey, corners, (5, 5), (-1, -1),
                                   (cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER, 30, 0.01))
        size = grey.shape[::-1]
        obj.append(board)
        img.append(corners)
    if len(obj) < 3:
        sys.exit(f"found the {cols}x{rows} board in {len(obj)} image(s); need at least 3")
    rms, k, dist, _, _ = cv2.calibrateCamera(obj, img, size, None, None)
    return {"camera_matrix": k.tolist(), "dist_coeffs": dist.ravel().tolist(), "image_size": list(size),
            "rms_px": rms, "images_used": len(obj)}


def main(argv=None):
    a = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    a.add_argument("out", help="calibration JSON to write")
    a.add_argument("images", nargs="+", help="checkerboard photos")
    a.add_argument("--cols", type=int, default=9, help="inner corners per row (default 9)")
    a.add_argument("--rows", type=int, default=6, help="inner corners per column (default 6)")
    a.add_argument("--square-m", type=float, default=0.025, help="printed square side in metres (default 0.025)")
    args = a.parse_args(argv)
    calib = calibrate(args.images, args.cols, args.rows, args.square_m)
    with open(args.out, "w") as f:
        json.dump(calib, f, indent=1)
    print(f"{calib['images_used']} of {len(args.images)} images used, rms {calib['rms_px']:.2f} px -> {args.out}")


if __name__ == "__main__":
    main()
