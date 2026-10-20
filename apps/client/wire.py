"""The car's payloads shared by the UDP and BLE clients: setpoint CDR, Status and Capabilities (src/device.rs).

test_wire.py checks them against the Rust golden vectors and codes.
"""
import struct
from typing import NamedTuple

# Status codes, indexed by the byte the car sends.
STATES = ("Open", "Claimed", "Armed", "Stopped")
REASONS = (None, "Stop", "LeaseExpired", "OwnerLost", "OutOfLimits")
ERRORS = (None, "Busy", "NotOwner", "NotReady", "Invalid")


def cdr(vx=0.0, vy=0.0, wz=0.0, sec=0, nanosec=0):
    """TwistStamped CDR (68 bytes): empty frame_id (NUL plus 3 zero pad bytes), then the twist."""
    return b"\x00\x01\x00\x00" + struct.pack("<iII", sec, nanosec, 1) + b"\x00" * 4 + struct.pack("<6d", vx, vy, 0, 0, 0, wz)


def ble_setpoint(seq, vx=0.0, vy=0.0, wz=0.0):
    """The BLE Setpoint value: `seq u32` LE, then the CDR (72 bytes)."""
    return struct.pack("<I", seq) + cdr(vx, vy, wz)


class Status(NamedTuple):
    """Shared state, stop reason and last refused owner ARM/STOP (NotReady, Invalid).
    Only successful owner control or release clears the error; setpoints and echo do not.
    Numeric codes remain version 1; Busy is a claimant reply, not a stored owner error."""
    state: str
    reason: str | None
    error: str | None

    @classmethod
    def decode(cls, b):
        if len(b) != 3 or b[0] >= len(STATES) or b[1] >= len(REASONS) or b[2] >= len(ERRORS):
            raise ValueError(f"bad status {bytes(b).hex()}")
        return cls(STATES[b[0]], REASONS[b[1]], ERRORS[b[2]])


class Capabilities(NamedTuple):
    version: int
    wheels_radius: float
    center_projection_sum: float
    max_vx: float
    max_vy: float
    max_wz: float
    lease_ms: int
    claim_ms: int
    reclaim_ms: int

    LEN = 27

    @classmethod
    def decode(cls, b):
        if len(b) != cls.LEN:
            raise ValueError(f"capabilities are {cls.LEN} bytes, got {len(b)}")
        return cls(*struct.unpack("<B5f3H", bytes(b)))
