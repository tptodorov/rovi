"""Run the board probe against the real product UDP simulator, with shorter pauses."""
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
    def run_probe(self, drop_bye=False):
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
                    # Isolate imports from the frozen BLE client's same-named wire module.
                    code = '''
import sys
from pathlib import Path
sys.path.insert(0, "firmware/bench")
import udp_stop_checks as probe
probe.LEAD_SECONDS = 0
probe.DRIVE_SECONDS = probe.PAUSE_SECONDS = 0.15
'''
                    if drop_bye:
                        code += '''
probe.MODES = ("LIMITS",)
send = probe.udp.Client.send
def without_bye(self, kind, *args, **kwargs):
    if kind != probe.udp.BYE:
        return send(self, kind, *args, **kwargs)
probe.udp.Client.send = without_bye
'''
                    code += 'probe.check(Path(sys.argv[1]))\n'
                    result = subprocess.run(
                        [sys.executable, "-c", code, str(path)], cwd=ROOT,
                        env={**os.environ, "ROVI_TARGET": f"127.0.0.1:{port}"},
                        capture_output=True, text=True, timeout=20,
                    )
                    return result, path.read_text()
                finally:
                    server.terminate()
                    server.wait(timeout=3)

    def test_udp_probe_against_product_simulator(self):
        result, text = self.run_probe()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("UDP stop checks: passed; released", result.stdout)
        for reason in ("Stop", "LeaseExpired", "OwnerLost", "OutOfLimits"):
            self.assertIn(f"ev=stop reason={reason}", text)

    def test_lost_cleanup_bye_cannot_report_released(self):
        result, _ = self.run_probe(drop_bye=True)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("release", result.stderr)


if __name__ == "__main__":
    unittest.main()
