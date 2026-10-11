import math, os, tempfile, unittest

import carlog, compare_track, plot_car

# idle 1 s, forward 1 s (0.5 m), rotate 1 s (2.5 rad), stop
LOG = "ev=boot ver=1 at_ms=0\nev=state state=Claimed wheels_pm=0,0,0,0 at_ms=100\n" \
      "ev=state state=Armed wheels_pm=250,250,250,250 at_ms=1100\n" \
      "ev=state state=Armed wheels_pm=-250,250,250,-250 at_ms=2100\nev=state state=Open wheels_pm=0,0,0,0 at_ms=3100\n"
START_US = 1_760_000_000_000_000  # camera epoch of the first wheel command


def observed(scale=1.0, turn=1.0, x0=0.3, y0=-0.2, h0=0.7, fps=30, lag_s=0.0):
    """Track rows (epoch_us, x_m, y_m, heading) of the commanded motion as a downward camera would see it: the car
    starts at (x0, y0) facing h0 in image axes. scale and turn shrink the real motion against the command."""
    pose = plot_car.commanded_path(carlog.segments(LOG)[-1])
    move = [p for p in pose if p[0] >= 1.0]  # from the first wheel command on
    rows = []
    for i in range(int(3.5 * fps) + 1):  # from 0.5 s before the motion until 1 s after it
        t = i / fps - 0.5
        tc = min(max(t - lag_s, 0.0), 2.0) + 1.0
        j = next(k for k in range(1, len(move)) if move[k][0] >= tc - 1e-9)
        (ta, xa, ya, tha), (tb, xb, yb, thb) = move[j - 1], move[j]
        f = (tc - ta) / (tb - ta)
        x, y, th = scale * (xa + f * (xb - xa)), scale * (ya + f * (yb - ya)), turn * (tha + f * (thb - tha))
        rows.append((START_US + round(t * 1e6), x0 + x * math.cos(h0) - y * math.sin(h0),
                     y0 + x * math.sin(h0) + y * math.cos(h0), math.atan2(math.sin(h0 + th), math.cos(h0 + th))))
    return rows


def write_track(rows, path, metric=True):
    with open(path, "w") as f:
        f.write("epoch_us,x_px,y_px,heading_rad,size_px" + (",x_m,y_m\n" if metric else "\n"))
        for t, x, y, h in rows:
            f.write(f"{t},{x * 1000:.2f},{-y * 1000:.2f},{h:.5f},60" + (f",{x:.5f},{y:.5f}\n" if metric else "\n"))


class Compare(unittest.TestCase):
    def setUp(self):
        self.cmd = compare_track.commanded(carlog.segments(LOG)[-1])

    def test_commanded_starts_at_the_first_wheel_command(self):
        self.assertAlmostEqual(self.cmd[0][0], 0.0)
        self.assertAlmostEqual(self.cmd[-1][0], 2.0)
        self.assertAlmostEqual(self.cmd[-1][1], 0.5)

    def test_motion_start_is_the_first_sample_that_moved(self):
        start = compare_track.motion_start_us(observed())
        self.assertLessEqual(abs(start - START_US), 40_000)  # within about one frame

    def test_observed_in_the_car_start_frame(self):
        obs = compare_track.observed(observed(), START_US)
        t, x, y, th = obs[-1]
        self.assertAlmostEqual(t, 3.0, places=3)
        self.assertAlmostEqual(x, 0.5, places=3)
        self.assertAlmostEqual(y, 0.0, places=3)
        self.assertAlmostEqual(th, 2.5, places=3)  # unwrapped, relative to the start heading

    def test_matching_motion_has_no_drift(self):
        d = compare_track.drift(self.cmd, compare_track.observed(observed(), START_US))
        self.assertAlmostEqual(d["cmd_dist_m"], 0.5, places=3)
        self.assertAlmostEqual(d["obs_dist_m"], 0.5, places=3)
        self.assertAlmostEqual(d["pos_err_m"], 0.0, places=3)
        self.assertAlmostEqual(d["heading_err_rad"], 0.0, places=3)

    def test_slip_shows_as_drift(self):
        d = compare_track.drift(self.cmd, compare_track.observed(observed(scale=0.8, turn=0.9), START_US))
        self.assertAlmostEqual(d["obs_dist_m"], 0.4, places=3)
        self.assertAlmostEqual(d["dist_err_m"], -0.1, places=3)
        self.assertAlmostEqual(d["pos_err_m"], 0.1, places=3)
        self.assertAlmostEqual(d["heading_err_rad"], -0.25, places=3)

    def test_track_needs_metric_columns(self):
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "t.csv")
            write_track(observed(), p, metric=False)
            with self.assertRaises(SystemExit):
                compare_track.read_track(p)

    def test_main_writes_png_and_prints_drift(self):
        with tempfile.TemporaryDirectory() as d:
            log, csv, png = (os.path.join(d, n) for n in ("car.log", "t.csv", "c.png"))
            with open(log, "w") as f:
                f.write(LOG)
            write_track(observed(lag_s=0.2), csv)
            d_ = compare_track.main([log, csv, png])
            self.assertGreater(os.path.getsize(png), 1000)
            self.assertAlmostEqual(d_["heading_err_rad"], 0.0, places=2)  # auto start absorbs the lag
            d_ = compare_track.main([log, csv, png, "--start-us", str(START_US)])
            self.assertLess(d_["heading_err_rad"], -0.1)  # a wrong start shows as drift


if __name__ == "__main__":
    unittest.main()
