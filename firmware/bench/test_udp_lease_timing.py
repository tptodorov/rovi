"""Exercise the timing probe against the product simulator on real UDP sockets."""
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[2]
SIMCAR = ROOT / "apps/simcar/target/debug/simcar"


class LeaseTiming(unittest.TestCase):
    def run_probe(self, extra=""):
        self.assertTrue(SIMCAR.is_file(), "build the product simcar before this test")
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as available:
            available.bind(("127.0.0.1", 0))
            port = available.getsockname()[1]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "car.log"
            with path.open("wb") as log:
                server = subprocess.Popen(
                    [str(SIMCAR), str(port), "1000", "10000", "5000"], stdout=log
                )
                try:
                    time.sleep(0.2)
                    self.assertIsNone(server.poll(), "simulator failed to start")
                    code = '''
import sys
from pathlib import Path
sys.path.insert(0, "firmware/bench")
import udp_lease_timing as probe
''' + extra + 'probe.check(Path(sys.argv[1]), trials=2)\n'
                    result = subprocess.run(
                        [sys.executable, "-c", code, str(path)], cwd=ROOT,
                        env={**os.environ, "ROVI_TARGET": f"127.0.0.1:{port}"},
                        capture_output=True, text=True, timeout=15,
                    )
                    return result, path.read_text()
                finally:
                    server.terminate()
                    server.wait(timeout=3)

    def test_probe_against_product_simulator(self):
        result, log = self.run_probe()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("UDP lease timing: passed", result.stdout)
        self.assertEqual(log.count("ev=stop reason=LeaseExpired"), 2)
        self.assertEqual(log.count("ev=released"), 2)

    def test_capture_delay_cannot_report_passed(self):
        result, log = self.run_probe('''
events = probe.carlog.events
def delayed(*args):
    probe.time.sleep(1.15)
    return events(*args)
probe.carlog.events = delayed
''')
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("exceeds lease+50", result.stderr)
        self.assertNotIn("UDP lease timing: passed", result.stdout)
        self.assertIn("ev=released", log)


if __name__ == "__main__":
    unittest.main()
