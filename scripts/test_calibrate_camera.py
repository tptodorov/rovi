"""Camera calibration on synthetic checkerboard views of a known pinhole camera.  python3 -m unittest discover -s scripts"""
import json, math, pathlib, tempfile, unittest

import cv2
import numpy as np

import calibrate_camera as cc

W, H, F = 1280, 720, 900.0
K = np.array([[F, 0, 640], [0, F, 360], [0, 0, 1]])
COLS, ROWS, SQ = 9, 6, 0.025  # inner corners, square side m


def board_image(rx, ry, rz, tx, ty, tz):
    """The checkerboard plane seen by camera K from a pose (rotation vector degrees, translation m)."""
    px = 40  # pixels per square on the flat board
    flat = np.full(((ROWS + 3) * px, (COLS + 3) * px), 255, np.uint8)
    for r in range(ROWS + 1):
        for c in range(COLS + 1):
            if (r + c) % 2 == 0:
                flat[(r + 1) * px:(r + 2) * px, (c + 1) * px:(c + 2) * px] = 0
    rot, _ = cv2.Rodrigues(np.radians([rx, ry, rz]))
    to_m = np.array([[SQ / px, 0, -(COLS + 3) * SQ / 2], [0, SQ / px, -(ROWS + 3) * SQ / 2], [0, 0, 1]])
    h = K @ np.column_stack([rot[:, 0], rot[:, 1], [tx, ty, tz]]) @ to_m
    return cv2.warpPerspective(flat, h, (W, H), borderValue=200)


POSES = [(0, 0, 0, 0, 0, 0.5), (20, 0, 5, 0.05, 0, 0.55), (-20, 10, 0, -0.05, 0.03, 0.5), (0, 25, -10, 0, -0.03, 0.6),
         (15, -20, 20, 0.04, 0.02, 0.55), (-10, -25, 0, -0.03, 0, 0.5)]


class Calibrate(unittest.TestCase):
    def test_recovers_the_focal_length_and_writes_json(self):
        with tempfile.TemporaryDirectory() as d:
            d = pathlib.Path(d)
            for i, p in enumerate(POSES):
                cv2.imwrite(str(d / f"{i}.png"), board_image(*p))
            cv2.imwrite(str(d / "blank.png"), np.full((H, W), 255, np.uint8))  # no board: skipped
            out = d / "calib.json"
            cc.main([str(out), *map(str, sorted(d.glob("*.png"))), "--cols", str(COLS), "--rows", str(ROWS)])
            calib = json.loads(out.read_text())
        self.assertEqual(calib["image_size"], [W, H])
        self.assertEqual(calib["images_used"], len(POSES))
        k = np.array(calib["camera_matrix"])
        self.assertAlmostEqual(k[0, 0] / F, 1, delta=0.02)
        self.assertAlmostEqual(k[1, 1] / F, 1, delta=0.02)
        self.assertLess(calib["rms_px"], 1.0)
        self.assertEqual(len(calib["dist_coeffs"]), 5)

    def test_too_few_boards_fails(self):
        with tempfile.TemporaryDirectory() as d:
            p = pathlib.Path(d) / "0.png"
            cv2.imwrite(str(p), board_image(*POSES[0]))
            with self.assertRaises(SystemExit):
                cc.calibrate([p], COLS, ROWS, SQ)


if __name__ == "__main__":
    unittest.main()
