"""The Python codecs against the frozen protocol vectors (add-rovi-protocol task 2.1).

KNOWN_GAPS lists today's differences from the vectors (design D8). Each one must stay a gap
until rovi-protocol replaces the codec; a gap that disappears fails here so the list shrinks."""
import json
import math
import struct
import unittest
from pathlib import Path

from apps.client.wire import Capabilities, Status, ble_setpoint, cdr
from apps.teleop.protocol import ClientError, capabilities, frame, network_text, parse

VECTORS = json.loads((Path(__file__).resolve().parents[3] / "crates/rovi-protocol/vectors.json").read_text())
# Python names (apps/client/wire.py) for the vector's ROS spellings (I4).
STATE = {"open": "Open", "claimed": "Claimed", "armed": "Armed", "stopped": "Stopped"}
NAME = {"none": None, "stop": "Stop", "lease-expired": "LeaseExpired", "owner-lost": "OwnerLost",
        "out-of-limits": "OutOfLimits", "busy": "Busy", "not-owner": "NotOwner", "not-ready": "NotReady",
        "invalid": "Invalid"}
KNOWN_GAPS = {
    # I2: apps/client Status.decode does not check that only Stopped carries a reason.
    ("status", "stopped-without-reason"), ("status", "reason-while-not-stopped"),
    # I3: network_text ignores the reserved byte.
    ("network", "reserved-nonzero"),
}


def cases(kind, message, encoding=None):
    return [v for v in VECTORS[kind] if v["message"] == message and (encoding is None or v["encoding"] == encoding)]


def f32(x):
    return struct.unpack("<f", struct.pack("<f", x))[0]


class Valid(unittest.TestCase):
    def test_status(self):
        for v in cases("valid", "status", "binary"):
            with self.subTest(v["name"]):
                s, want = Status.decode(bytes.fromhex(v["hex"])), v["value"]
                self.assertEqual(s, (STATE[want["state"]], NAME[want["reason"]], NAME[want["error"]]))

    def test_capabilities(self):
        for v in cases("valid", "capabilities", "binary"):
            with self.subTest(v["name"]):
                c = Capabilities.decode(bytes.fromhex(v["hex"]))
                want = v["value"]
                self.assertEqual(c.version, want["version"])
                self.assertEqual(c.center_projection_sum, f32(want["center_projection_sum"]))
                for key in ("wheels_radius", "max_vx", "max_vy", "max_wz"):
                    self.assertEqual(getattr(c, key), f32(want[key]))
                for key in ("lease_ms", "claim_ms", "reclaim_ms"):
                    self.assertEqual(getattr(c, key), want[key])

    def test_network_text_is_the_ros_text(self):
        ros = {v["name"]: v for v in cases("valid", "network", "ros")}
        for v in cases("valid", "network", "binary"):
            if v["name"] in ros:
                with self.subTest(v["name"]):
                    text = bytes.fromhex(ros[v["name"]]["hex"])[8:-1].decode()
                    self.assertEqual(network_text(bytes.fromhex(v["hex"])), text)

    def test_setpoint_encoders(self):
        for v in cases("valid", "setpoint", "cdr"):
            with self.subTest(v["name"]):
                s = v["value"]
                self.assertEqual(cdr(s["vx"], s["vy"], s["wz"], s["sec"], s["nanosec"]).hex(), v["hex"])
        for v in cases("valid", "ble-setpoint"):
            with self.subTest(v["name"]):
                s = v["value"]["setpoint"]
                self.assertEqual(ble_setpoint(v["value"]["seq"], s["vx"], s["vy"], s["wz"]).hex(), v["hex"])

    def test_udp_frames(self):
        for v in cases("valid", "udp-frame"):
            with self.subTest(v["name"]):
                f, payload = v["value"], bytes.fromhex(v["value"]["payload"])
                self.assertEqual(frame(f["kind"], f["session"], f["sequence"], payload).hex(), v["hex"])
                self.assertEqual(parse(bytes.fromhex(v["hex"])), (f["kind"], f["session"], f["sequence"], payload))


class Invalid(unittest.TestCase):
    def rejects(self, v, decode):
        try:
            decode(bytes.fromhex(v["hex"]))
        except (ValueError, ClientError, struct.error, IndexError):
            return True
        return False

    def check(self, message, decode):
        for v in cases("invalid", message, "binary"):
            with self.subTest(v["name"]):
                if (message, v["name"]) in KNOWN_GAPS:
                    self.assertFalse(self.rejects(v, decode), "gap closed: remove it from KNOWN_GAPS")
                else:
                    self.assertTrue(self.rejects(v, decode))

    def test_status(self):
        self.check("status", Status.decode)

    def test_capabilities(self):
        self.check("capabilities", capabilities)

    def test_network(self):
        def decode(b):
            if len(b) != 8 or network_text(b) == "network information not understood":
                raise ValueError(b.hex())
        self.check("network", decode)

    def test_udp_frames(self):
        self.check("udp-frame", parse)


if __name__ == "__main__":
    unittest.main()
