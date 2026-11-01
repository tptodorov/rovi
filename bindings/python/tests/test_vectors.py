"""The Python binding against crates/rovi-protocol/vectors.json (add-rovi-protocol task 3.1).

Every valid case decodes to its value and encodes back to its bytes; every invalid case raises
DecodeError with the crate's error kind. Run after installing the wheel:
`python3 -m unittest discover -s bindings/python/tests`."""
import json
import struct
import unittest
from pathlib import Path

import rovi_protocol as rp

VECTORS = json.loads((Path(__file__).resolve().parents[3] / "crates/rovi-protocol/vectors.json").read_text())
FLOATS = ("wheels_radius", "center_projection_sum", "max_vx", "max_vy", "max_wz")
FROM_CAR = {2, 7, 8}  # HELLO_ACK, STATUS, BUSY; ECHO goes both ways, INFO by payload


def f32(x):
    return struct.unpack("<f", struct.pack("<f", x))[0]


def caps(v):
    return {k: f32(x) if k in FLOATS else x for k, x in v.items()}


def from_car(b):
    """The direction a UDP frame travels, from its kind and payload."""
    kind = b[3] if len(b) > 3 else 0
    return kind in FROM_CAR or kind == 9 and len(b) > 12 + rp.ECHO_MAX or kind == 11 and len(b) > 12


CODECS = {
    ("status", "binary"): (rp.status_decode, rp.status_encode, lambda v: v),
    ("status", "ros"): (rp.status_decode_ros, rp.status_encode_ros, lambda v: v),
    ("capabilities", "binary"): (rp.capabilities_decode, rp.capabilities_encode, caps),
    ("capabilities", "ros"): (rp.capabilities_decode_ros, rp.capabilities_encode_ros, caps),
    ("network", "binary"): (rp.network_decode, rp.network_encode, lambda v: v),
    ("network", "ros"): (rp.network_decode_ros, rp.network_encode_ros, lambda v: v),
    ("setpoint", "cdr"): (rp.setpoint_decode, rp.setpoint_encode, lambda v: v),
}


class Valid(unittest.TestCase):
    def test_every_value_round_trips(self):
        seen = set()
        for v in VECTORS["valid"]:
            key, b = (v["message"], v["encoding"]), bytes.fromhex(v["hex"])
            with self.subTest(f"{key} {v['name']}"):
                seen.add(key)
                if key in CODECS:
                    decode, encode, expect = CODECS[key]
                    self.assertEqual(decode(b), expect(v["value"]))
                    self.assertEqual(encode(v["value"]), b)
                elif key == ("setpoint", "cdr-decode-only"):
                    self.assertEqual(rp.setpoint_decode(b), v["value"])
                elif key == ("ble-setpoint", "binary"):
                    seq, sp = v["value"]["seq"], v["value"]["setpoint"]
                    self.assertEqual(rp.ble_setpoint_decode(b), (seq, sp))
                    self.assertEqual(rp.ble_setpoint_encode(seq, sp), b)
                elif key == ("ble-control", "binary"):
                    self.assertEqual(rp.control_decode(b), v["value"])
                    self.assertEqual(rp.control_encode(v["value"]), b)
                elif key == ("udp-frame", "binary"):
                    decode, encode = (rp.udp_decode_from_car, rp.udp_encode_from_car) if from_car(b) \
                        else (rp.udp_decode_to_car, rp.udp_encode_to_car)
                    frame = decode(b)
                    self.assertEqual((frame["session"], frame["seq"]), (v["value"]["session"], v["value"]["sequence"]))
                    self.assertEqual(encode(frame), b)
                else:
                    self.fail(f"no binding check for {key}")
        self.assertEqual(seen, {k for k in CODECS} | {("setpoint", "cdr-decode-only"), ("ble-setpoint", "binary"),
                                                     ("ble-control", "binary"), ("udp-frame", "binary")})


class Invalid(unittest.TestCase):
    def test_every_malformed_case_names_the_crates_error(self):
        decoders = {"status": rp.status_decode, "capabilities": rp.capabilities_decode,
                    "network": rp.network_decode, "setpoint": rp.setpoint_decode}
        for v in VECTORS["invalid"]:
            b = bytes.fromhex(v["hex"])
            decode = decoders.get(v["message"]) or (rp.udp_decode_from_car if from_car(b) else rp.udp_decode_to_car)
            with self.subTest(f"{v['message']} {v['name']}"):
                with self.assertRaises(rp.DecodeError) as e:
                    decode(b)
                self.assertEqual(e.exception.args[0], v["error"])


class Api(unittest.TestCase):
    def test_constants_and_client_minimum(self):
        self.assertEqual(rp.UDP_PORT, 7777)
        self.assertEqual(rp.BLE_UUIDS["network"], "d47a0016-45b2-4d19-8db0-6bd87e9c0001")
        caps_value = rp.capabilities_decode(bytes.fromhex(next(
            v["hex"] for v in VECTORS["valid"] if v["message"] == "capabilities" and v["encoding"] == "binary")))
        self.assertTrue(rp.meets_client_minimum(caps_value))
        self.assertFalse(rp.meets_client_minimum(dict(caps_value, lease_ms=rp.MIN_LEASE_MS - 1)))

    def test_encoders_refuse_what_decoders_refuse(self):
        with self.assertRaises(rp.EncodeError):
            rp.setpoint_encode({"vx": float("nan")})
        with self.assertRaises(rp.EncodeError):
            rp.status_encode({"state": "stopped", "reason": "none", "error": "none"})
        with self.assertRaises(rp.EncodeError):
            rp.udp_encode_to_car({"session": 1, "seq": 1, "message": {"kind": "echo", "data": bytes(33)}})


if __name__ == "__main__":
    unittest.main()
