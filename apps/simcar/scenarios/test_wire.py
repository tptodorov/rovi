import pathlib, re, struct, unittest

import wire

SRC = pathlib.Path(__file__).resolve().parents[3] / "crates/rovi-core/src"


def golden(name):
    """A CDR golden vector captured from ROS 2 Jazzy, read from the Rust tests in src/setpoint.rs."""
    return bytes.fromhex(re.search(rf'const {name}: &str = "([0-9a-f]+)"', (SRC / "setpoint.rs").read_text()).group(1))


class Setpoint(unittest.TestCase):
    def test_cdr_matches_ros_bytes_up_to_padding(self):
        ros, ours = golden("EMPTY_FRAME"), wire.cdr(0.1)
        self.assertEqual(len(ours), 68)
        self.assertEqual(ours[:17], ros[:17])
        self.assertEqual(ours[20:], ros[20:])
        self.assertEqual(ours[17:20], b"\0\0\0", "padding is zeroed, as the Rust encoder does")

    def test_cdr_carries_stamp_and_the_plane_components(self):
        b = wire.cdr(0.5, -0.25, 1.5, sec=1, nanosec=2)
        self.assertEqual(struct.unpack_from("<iI", b, 4), (1, 2))
        self.assertEqual(struct.unpack_from("<6d", b, 20), (0.5, -0.25, 0, 0, 0, 1.5))
        # the twist sits where it does in the ROS capture with stamp (1, 2)
        self.assertEqual(b[4:12], golden("BASE_LINK")[4:12])

    def test_ble_setpoint_is_seq_then_cdr(self):
        b = wire.ble_setpoint(0x01020304, 0.1)
        self.assertEqual(len(b), 72)
        self.assertEqual(b[:4], bytes([4, 3, 2, 1]))
        self.assertEqual(b[4:], wire.cdr(0.1))


class Status(unittest.TestCase):
    def test_decode(self):
        self.assertEqual(wire.Status.decode(b"\x03\x02\x00"), ("Stopped", "LeaseExpired", None))
        self.assertEqual(wire.Status.decode(bytearray([1, 0, 3])), ("Claimed", None, "NotReady"))
        self.assertEqual(wire.Status.decode(bytes([2, 0, 0])).state, "Armed")

    def test_unknown_codes_and_lengths_are_rejected(self):
        for b in (b"\x09\x00\x00", b"\x00\x09\x00", b"\x00\x00\x09", b"\x00\x00"):
            with self.assertRaises(ValueError):
                wire.Status.decode(b)

    def test_codes_match_the_firmware(self):
        src = (SRC / "device.rs").read_text()
        self.assertEqual(int(re.search(r"STATUS_LEN: usize = (\d+)", src).group(1)), 3)
        for name, code in re.findall(r"StopReason::(\w+) => (\d+)", src):
            self.assertEqual(wire.REASONS[int(code)], name)
        for name, code in re.findall(r"Some\(Error::(\w+)\) => (\d+)", src):
            self.assertEqual(wire.ERRORS[int(code)], name)
        for name, code in re.findall(r"State::(\w+)(?:\(r\))? => \((\d+),", src):
            self.assertEqual(wire.STATES[int(code)], name)


class Capabilities(unittest.TestCase):
    def test_decode(self):
        b = bytes([1]) + struct.pack("<5f", 0.05, 0.2, 1.0, 1.0, 3.0) + struct.pack("<3H", 1000, 10000, 5000)
        c = wire.Capabilities.decode(b)
        self.assertEqual((c.version, c.lease_ms, c.claim_ms, c.reclaim_ms), (1, 1000, 10000, 5000))
        self.assertAlmostEqual(c.wheels_radius, 0.05)
        self.assertAlmostEqual(c.max_wz, 3.0)
        with self.assertRaises(ValueError):
            wire.Capabilities.decode(b[:-1])

    def test_length_matches_the_firmware(self):
        src = (SRC / "device.rs").read_text()
        self.assertEqual(int(re.search(r"CAPABILITIES_LEN: usize = (\d+)", src).group(1)), wire.Capabilities.LEN)


if __name__ == "__main__":
    unittest.main()
