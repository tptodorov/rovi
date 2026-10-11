import math, os, tempfile, unittest

import carlog, plot_car

LOG = "ev=boot ver=1 at_ms=0\nev=state state=Armed wheels_pm=250,250,250,250 at_ms=100\n" \
      "ev=state state=Armed wheels_pm=-250,250,250,-250 at_ms=1100\nev=state state=Open wheels_pm=0,0,0,0 at_ms=2100\n"


class CommandedPath(unittest.TestCase):
    def test_forward_then_rotate(self):
        pose = plot_car.commanded_path(carlog.segments(LOG)[-1])
        t, x, y, th = pose[-1]
        self.assertAlmostEqual(t, 2.0)
        self.assertAlmostEqual(x, 0.5)  # 0.25 * 40 rad/s * 0.05 m for 1 s
        self.assertAlmostEqual(y, 0.0)
        self.assertAlmostEqual(th, 2.5)  # 0.05 / 0.2 * 10 rad/s for 1 s
        self.assertEqual([round(p[0], 3) for p in pose], [0.0, 1.0, 2.0, 2.0])

    def test_no_state_events(self):
        self.assertIsNone(plot_car.commanded_path(carlog.segments("ev=boot at_ms=0\n")[-1]))

    def test_main_writes_png(self):
        with tempfile.TemporaryDirectory() as d:
            log, png = os.path.join(d, "car.log"), os.path.join(d, "p.png")
            open(log, "w").write(LOG)
            plot_car.main(log, png)
            self.assertGreater(os.path.getsize(png), 1000)
