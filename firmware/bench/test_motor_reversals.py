"""Real-socket reversal probe checks using the opt-in product coordinator harness."""
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]


class Reversals(unittest.TestCase):
    def replay_zero_hold(self, late_claimed=False):
        sys.path.insert(0, str(ROOT / "firmware/bench"))
        import motor_reversals as probe

        # Minimal trace from the board: a valid pre-ARM Claimed event remains in
        # the file on every poll. Rechecking it must not invent a later transition.
        trace = ("ev=state state=Claimed wheels_pm=0,0,0,0 at_ms=16509\n"
                 "ev=state state=Armed wheels_pm=0,0,0,0 at_ms=16521\n")
        if late_claimed:
            trace += "ev=state state=Claimed wheels_pm=0,0,0,0 at_ms=16540\n"
        client = SimpleNamespace(session=17, send=lambda *args: None,
                                 hello=lambda: (probe.udp.ACK, 17, b""))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "car.log"
            path.write_text(trace)
            with patch.object(probe, "HOLD_SECONDS", 0.15), \
                 patch.object(probe.time, "monotonic", side_effect=[0, 0, .05, .10, .20]), \
                 patch.object(probe.time, "sleep"):
                probe.stage(client, path, 0, [0] * 4, offset=0)

    def test_claimed_prefix_is_valid_on_every_poll(self):
        self.replay_zero_hold()

    def test_claimed_after_armed_is_still_rejected(self):
        with self.assertRaisesRegex(AssertionError, "Claimed"):
            self.replay_zero_hold(late_claimed=True)

    def run_probe(self, bench=True, extra=""):
        binary = ROOT / "apps/simcar/target" / ("motor-bench/debug/simcar" if bench else "debug/simcar")
        self.assertTrue(binary.is_file(), "build normal and motor-bench simcars first")
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
            sock.bind(("127.0.0.1", 0))
            port = sock.getsockname()[1]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "car.log"
            with path.open("wb") as output:
                server = subprocess.Popen([str(binary), str(port)], stdout=output)
                try:
                    time.sleep(0.2)
                    self.assertIsNone(server.poll(), "simcar did not start")
                    code = '''
import sys
from pathlib import Path
sys.path.insert(0, "firmware/bench")
import motor_reversals as probe
probe.LEAD_SECONDS = 0
probe.HOLD_SECONDS = 0.15
''' + extra + '\nprobe.check(Path(sys.argv[1]))\n'
                    result = subprocess.run(
                        [sys.executable, "-c", code, str(path)], cwd=ROOT,
                        env={**os.environ, "ROVI_TARGET": f"127.0.0.1:{port}"},
                        capture_output=True, text=True, timeout=15,
                    )
                    return result, path.read_text()
                finally:
                    server.terminate()
                    server.wait(timeout=3)

    def test_full_batch_and_dropped_first_cleanup_bye(self):
        result, log = self.run_probe(extra='''
send = probe.udp.Client.send
dropped = False
def drop_first_bye(self, kind, *args, **kwargs):
    global dropped
    if kind == probe.udp.BYE and not dropped:
        dropped = True
        return
    return send(self, kind, *args, **kwargs)
probe.udp.Client.send = drop_first_bye
''')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("Ten direct reversals: serial/session checks passed", result.stdout)
        self.assertEqual(log.count("wheels_pm=-1000,-1000,-1000,-1000"), 5)
        self.assertEqual(log.count("wheels_pm=1000,1000,1000,1000"), 6)
        self.assertEqual(log.count("ev=boot"), 1)
        self.assertEqual(log.count("ev=claimed"), 1)
        self.assertIn("ev=released", log)

    def test_wrong_motor_evidence_fails_and_releases(self):
        result, log = self.run_probe(extra="probe.FORWARD_PM = 999")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("wrong motor demand", result.stderr)
        self.assertIn("ev=released", log)

    def test_normal_image_refused_before_claim(self):
        result, log = self.run_probe(bench=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("bench image marker absent", result.stderr)
        self.assertNotIn("ev=claimed", log)


if __name__ == "__main__":
    unittest.main()
