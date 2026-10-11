"""Marker tracking on synthetic frames: no camera, no car.  python3 -m unittest discover -s scripts"""
import csv, math, pathlib, tempfile, unittest

import cv2
import numpy as np

import track_marker as tm

W, H, SIDE = 1280, 720, 120  # frame size and marker side in pixels


def frame(markers=(), size=SIDE):
    """A white frame with ArUco DICT_4X4_50 markers drawn at (id, cx, cy, heading_rad); heading is counter-clockwise as seen."""
    img = np.full((H, W), 255, np.uint8)
    for mid, cx, cy, heading in markers:
        m = cv2.aruco.generateImageMarker(cv2.aruco.getPredefinedDictionary(cv2.aruco.DICT_4X4_50), mid, size)
        q = size // 4  # white quiet zone around the marker, as on a print
        tile = cv2.copyMakeBorder(m, q, q, q, q, cv2.BORDER_CONSTANT, value=255)
        c = tile.shape[0] / 2
        rot = cv2.getRotationMatrix2D((c, c), math.degrees(heading), 1.0)
        rot[:, 2] += (cx - c, cy - c)
        mask = cv2.warpAffine(np.full(tile.shape, 255, np.uint8), rot, (W, H))
        warped = cv2.warpAffine(tile, rot, (W, H), borderValue=255)
        img[mask > 0] = warped[mask > 0]
    return cv2.cvtColor(img, cv2.COLOR_GRAY2BGR)


def angle_diff(a, b):
    return math.atan2(math.sin(a - b), math.cos(a - b))


class Detect(unittest.TestCase):
    def test_position_and_heading_recovered(self):
        for cx, cy, deg in [(640, 360, 0), (300, 200, 30), (1000, 500, -75), (500, 550, 135), (800, 150, 180)]:
            with self.subTest(cx=cx, cy=cy, deg=deg):
                d = tm.detect(frame([(0, cx, cy, math.radians(deg))]))
                self.assertIsNotNone(d)
                self.assertAlmostEqual(d.x_px, cx, delta=1.5)
                self.assertAlmostEqual(d.y_px, cy, delta=1.5)
                self.assertLess(abs(angle_diff(d.heading_rad, math.radians(deg))), math.radians(1.5))
                self.assertAlmostEqual(d.size_px, SIDE, delta=2)

    def test_picks_the_requested_id(self):
        img = frame([(3, 300, 300, 0.0), (0, 900, 400, 0.5)])
        self.assertAlmostEqual(tm.detect(img).x_px, 900, delta=1.5)
        self.assertAlmostEqual(tm.detect(img, marker_id=3).x_px, 300, delta=1.5)

    def test_missing_or_partial_marker_gives_none(self):
        self.assertIsNone(tm.detect(frame()), "blank frame")
        self.assertIsNone(tm.detect(frame([(5, 640, 360, 0.0)])), "only another id")
        self.assertIsNone(tm.detect(frame([(0, W - 10, 360, 0.0)])), "marker mostly off the frame")

    def test_calibration_undistorts_corners(self):
        calib = {"camera_matrix": [[1000, 0, 640], [0, 1000, 360], [0, 0, 1]], "dist_coeffs": [0, 0, 0, 0, 0]}
        d = tm.detect(frame([(0, 400, 300, 0.3)]), calib=calib)  # zero distortion: same answer as without
        self.assertAlmostEqual(d.x_px, 400, delta=1.5)
        self.assertLess(abs(angle_diff(d.heading_rad, 0.3)), math.radians(1.5))


class Frames(unittest.TestCase):
    def test_frame_time_from_name(self):
        self.assertEqual(tm.frame_time_us(pathlib.Path("cam/frames/1760000000123456.jpg")), 1760000000123456)
        self.assertIsNone(tm.frame_time_us(pathlib.Path("sheet.jpg")))
        self.assertIsNone(tm.frame_time_us(pathlib.Path("12.png")))

    def test_track_writes_rows_in_time_order_and_skips_bad_frames(self):
        with tempfile.TemporaryDirectory() as d:
            d = pathlib.Path(d)
            cv2.imwrite(str(d / "2000.jpg"), frame([(0, 700, 360, 0.0)]))
            cv2.imwrite(str(d / "1000.jpg"), frame([(0, 600, 360, 0.0)]))
            cv2.imwrite(str(d / "1500.jpg"), frame())  # car out of view
            (d / "1700.jpg").write_bytes(b"not a jpeg")  # cut-off file
            cv2.imwrite(str(d / "sheet.jpg"), frame([(0, 640, 360, 0.0)]))  # not a frame name
            rows = tm.track(d)
            self.assertEqual([r.epoch_us for r in rows], [1000, 2000])
            self.assertAlmostEqual(rows[1].x_px, 700, delta=1.5)

    def test_csv_with_metric_scaling(self):
        rows = [tm.Row(1000, 100.0, 50.0, 0.1, 120.0), tm.Row(2000, 340.0, 50.0, 0.2, 120.0)]
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "t.csv"
            tm.write_csv(rows, out, side_m=0.06)
            got = list(csv.DictReader(out.read_text().splitlines()))
        self.assertEqual(list(got[0]), ["epoch_us", "x_px", "y_px", "heading_rad", "size_px", "x_m", "y_m"])
        self.assertEqual(got[1]["epoch_us"], "2000")
        self.assertAlmostEqual(float(got[1]["x_m"]) - float(got[0]["x_m"]), 0.12, places=6)  # 240 px at 0.06 m / 120 px
        self.assertAlmostEqual(float(got[0]["y_m"]), -0.025, places=6)  # y_m points up the image

    def test_csv_without_scaling_has_pixel_columns_only(self):
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "t.csv"
            tm.write_csv([tm.Row(1, 1.0, 2.0, 0.0, 10.0)], out)
            self.assertEqual(out.read_text().splitlines()[0], "epoch_us,x_px,y_px,heading_rad,size_px")


if __name__ == "__main__":
    unittest.main()
