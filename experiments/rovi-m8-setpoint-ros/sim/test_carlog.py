import os, tempfile, unittest

import carlog


def check(text):
    with tempfile.NamedTemporaryFile("w", delete=False) as f:
        f.write(text)
    try:
        return carlog.check(f.name)
    finally:
        os.unlink(f.name)


class CarLog(unittest.TestCase):
    def test_parse(self):
        e = carlog.parse("I (123) ev=state state=Armed wheels=0.25,-0.25,0,0.1 at_ms=99\r")
        self.assertEqual((e["state"], e["wheels"], e["at_ms"]), ("Armed", [0.25, -0.25, 0.0, 0.1], 99))
        self.assertIsNone(carlog.parse("ev=stop reason=Stop at_"), "cut-short line")
        self.assertIsNone(carlog.parse("wifi AP on at_ms=5"))

    def test_clean_log(self):
        problems, boots, counts = check("simcar listening on udp/1\nev=claimed source=Udp at_ms=1\nev=released at_ms=2\n")
        self.assertEqual((problems, boots, counts["claimed"]), ([], 1, 1))

    def test_double_claim_and_stray_release(self):
        problems, _, _ = check("ev=claimed source=Udp at_ms=1\nev=claimed source=Ble at_ms=2\nM8 board-only\nev=released at_ms=3\n")
        self.assertEqual(len(problems), 2)

    def test_boot_resets_ownership(self):
        problems, boots, _ = check("ev=claimed source=Udp at_ms=1\nM8 board-only AP\nev=claimed source=Udp at_ms=1\n")
        self.assertEqual((problems, boots), ([], 1))

    def test_panic(self):
        problems, _, _ = check("====== PANIC ======\n")
        self.assertEqual(len(problems), 1)


if __name__ == "__main__":
    unittest.main()
