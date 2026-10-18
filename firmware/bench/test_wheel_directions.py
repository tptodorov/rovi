"""Run the wheel-direction probe against the real product UDP simulator, with short timings."""
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


class SimProbe(unittest.TestCase):
    def run_probe(self, extra=""):
        self.assertTrue(SIMCAR.is_file(), "build the product simcar before this test")
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as port_probe:
            port_probe.bind(("127.0.0.1", 0))
            port = port_probe.getsockname()[1]
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
import wheel_directions as probe
probe.LEAD_SECONDS = 0
probe.DRIVE_SECONDS = probe.PAUSE_SECONDS = 0.2
''' + extra + 'probe.check(Path(sys.argv[1]))\n'
                    result = subprocess.run(
                        [sys.executable, "-c", code, str(path)], cwd=ROOT,
                        env={**os.environ, "ROVI_TARGET": f"127.0.0.1:{port}"},
                        capture_output=True, text=True, timeout=20,
                    )
                    return result, path.read_text()
                finally:
                    server.terminate()
                    server.wait(timeout=3)

    def test_probe_against_product_simulator(self):
        result, text = self.run_probe()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("serial signs passed; released", result.stdout)
        self.assertIn("ev=released", text)

    def test_an_inverted_wheel_fails_the_probe(self):
        # Expect FL reversed on "strafe left": the real (correct) signs must then be rejected.
        result, _ = self.run_probe('''
name, vx, vy, wz, signs = probe.MOVES[2]
probe.MOVES = (probe.MOVES[0], probe.MOVES[1], (name, vx, vy, wz, (1,) + signs[1:]))
''')
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("strafe left", result.stderr)


if __name__ == "__main__":
    unittest.main()
