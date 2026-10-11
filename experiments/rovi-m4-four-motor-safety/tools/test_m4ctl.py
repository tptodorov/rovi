import os
import pty
import subprocess
import sys
import tempfile
import tty
import unittest

sys.path.insert(0, os.path.dirname(__file__))
import m4ctl  # noqa: E402

HERE = os.path.dirname(__file__)


class Steps(unittest.TestCase):
    def test_kinds(self):
        self.assertEqual(
            m4ctl.steps(["en", "w 0.3 0 0 -0.3 for 2000", "wait 500", "hang high"]),
            [
                ("send", "en", 0),
                ("stream", "w 0.3 0 0 -0.3", 2000),
                ("wait", None, 500),
                ("send", "hang high", 0),
            ],
        )

    def test_bad_numbers_fail(self):
        with self.assertRaises(ValueError):
            m4ctl.steps(["wait soon"])


class OverPty(unittest.TestCase):
    """The tool against a pseudo-terminal standing in for the board: what it sends, what it logs."""

    def test_streams_and_logs(self):
        master, slave = pty.openpty()
        tty.setraw(slave)  # no echo of what the 'board' prints
        with tempfile.TemporaryDirectory() as d:
            log = os.path.join(d, "car.log")
            os.write(master, b"ev=boot ver=0.1.0 reset=ChipPowerOn at_ms=0\n")
            p = subprocess.run(
                [sys.executable, os.path.join(HERE, "m4ctl.py"), os.ttyname(slave), log,
                 "en", "w 0.1 0 0 0 for 200", "--tail-ms", "200"],
                capture_output=True, timeout=10,
            )
            self.assertEqual(p.returncode, 0, p.stderr)
            sent = os.read(master, 4096).decode()
            lines = sent.splitlines()
            self.assertEqual(lines[0], "en")
            self.assertGreaterEqual(lines.count("w 0.1 0 0 0"), 3)  # 20 Hz for 200 ms
            with open(log) as f:
                self.assertIn("ev=boot", f.read())
        os.close(master)
        os.close(slave)


if __name__ == "__main__":
    unittest.main()
