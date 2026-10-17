import os, pathlib, re, tempfile, unittest

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
        e = carlog.parse("I (123) ev=state state=Armed wheels_pm=250,-250,0,100 at_ms=99\r")
        self.assertEqual((e["state"], e["wheels_pm"], e["at_ms"]), ("Armed", [250, -250, 0, 100], 99))
        self.assertEqual(carlog.parse("ev=ble link=up id=1 mtu=75 at_ms=5")["mtu"], 75)
        self.assertIsNone(carlog.parse("ev=stop reason=Stop at_"), "cut-short line")
        self.assertIsNone(carlog.parse("wifi AP on at_ms=5"))

    def test_unknown_keys_are_kept_so_events_can_grow(self):
        self.assertEqual(carlog.parse("ev=claimed source=Udp new_key=7 at_ms=1")["new_key"], 7)

    def test_clean_log(self):
        problems, boots, counts = check("ev=boot ver=1 at_ms=0\nev=claimed source=Udp at_ms=1\nev=released at_ms=2\n")
        self.assertEqual((problems, boots, counts["claimed"]), ([], 1, 1))

    def test_double_claim_and_stray_release(self):
        problems, _, _ = check("ev=boot at_ms=0\nev=claimed source=Udp at_ms=1\nev=claimed source=Ble at_ms=2\nev=boot at_ms=0\nev=released at_ms=3\n")
        self.assertEqual(len(problems), 2)

    def test_boot_resets_ownership(self):
        problems, boots, _ = check("ev=boot at_ms=0\nev=claimed source=Udp at_ms=1\nev=boot at_ms=0\nev=claimed source=Udp at_ms=1\n")
        self.assertEqual((problems, boots), ([], 2))

    def test_faults_losses_unknown_events_and_panics(self):
        problems, _, _ = check("ev=error src=wifi_ap at_ms=1\nev=dropped n=3 at_ms=2\nev=nope at_ms=3\n====== PANIC ======\n")
        self.assertEqual(len(problems), 4)

    def test_ev_predicate_matches_name_and_fields(self):
        e = carlog.parse("ev=stop reason=LeaseExpired at_ms=7")
        self.assertTrue(carlog.ev("stop")(e))
        self.assertTrue(carlog.ev("stop", reason="LeaseExpired")(e))
        self.assertFalse(carlog.ev("stop", reason="Stop")(e))
        self.assertFalse(carlog.ev("state")(e))

    def test_wait_finds_events_after_the_offset_only(self):
        with tempfile.NamedTemporaryFile("w") as f:
            f.write("ev=released at_ms=1\n")
            f.flush()
            off = os.path.getsize(f.name)
            self.assertEqual(carlog.wait(f.name, off, carlog.ev("released"), 0.1), (False, []))
            f.write("ev=claimed source=Ble at_ms=2\n")
            f.flush()
            found, seen = carlog.wait(f.name, off, carlog.ev("claimed", source="Ble"), 0.1)
            self.assertTrue(found)
            self.assertEqual(seen[-1]["source"], "Ble")

    def test_catalog_matches_the_firmware_events(self):
        src = (pathlib.Path(__file__).resolve().parents[3] / "crates/rovi-core/src/events.rs").read_text()
        self.assertEqual(set(re.findall(r'"ev=([a-z_]+)', src)), carlog.CATALOG)


if __name__ == "__main__":
    unittest.main()
