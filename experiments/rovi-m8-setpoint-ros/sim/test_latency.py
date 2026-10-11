"""Tests for sim/latency.py's statistics and probe payload (no car needed)."""
import unittest

import latency


class Stats(unittest.TestCase):
    def test_median_p95_max_and_loss(self):
        s = latency.stats([float(x) for x in range(1, 101)], sent=200)
        self.assertEqual((s["sent"], s["count"], s["loss_pct"]), (200, 100, 50.0))
        self.assertEqual((s["median_ms"], s["p95_ms"], s["max_ms"]), (50.5, 95.0, 100.0))

    def test_p95_is_nearest_rank(self):
        self.assertEqual(latency.stats([1.0, 2.0, 3.0], sent=3)["p95_ms"], 3.0)

    def test_nothing_received(self):
        s = latency.stats([], sent=5)
        self.assertEqual((s["count"], s["loss_pct"], s["median_ms"]), (0, 100.0, None))

    def test_line_is_one_parseable_row(self):
        line = latency.line("udp", latency.stats([1.0, 3.0], sent=2))
        self.assertEqual(line, "udp sent=2 count=2 loss_pct=0.0 median_ms=2.000 p95_ms=3.000 max_ms=3.000")


class Probe(unittest.TestCase):
    def test_probe_round_trips_and_fits_the_echo_limit(self):
        p = latency.probe(7, 123456789)
        self.assertLessEqual(len(p), latency.ECHO_MAX)
        self.assertEqual(latency.unprobe(p), (7, 123456789))

    def test_foreign_payload_is_ignored(self):
        self.assertIsNone(latency.unprobe(b"xyz"))


if __name__ == "__main__":
    unittest.main()
