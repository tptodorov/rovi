"""The car's payloads for the UDP and BLE clients: setpoint CDR, Status and Capabilities.

A thin adapter over the `rovi_protocol` binding of the rovi-protocol crate (ADR-0017): layouts and
validation live only there. Names stay in this module's spelling until the clients adopt the
crate's (add-rovi-protocol design I4).
"""
from typing import NamedTuple

import rovi_protocol as rp

# Status codes, indexed by the byte the car sends.
STATES = ("Open", "Claimed", "Armed", "Stopped")
REASONS = (None, "Stop", "LeaseExpired", "OwnerLost", "OutOfLimits")
ERRORS = (None, "Busy", "NotOwner", "NotReady", "Invalid")


def _camel(name):
    return None if name == "none" else "".join(w.capitalize() for w in name.split("-"))


def cdr(vx=0.0, vy=0.0, wz=0.0, sec=0, nanosec=0):
    """TwistStamped CDR (68 bytes) with an empty frame id."""
    return rp.setpoint_encode({"vx": vx, "vy": vy, "wz": wz, "sec": sec, "nanosec": nanosec})


def ble_setpoint(seq, vx=0.0, vy=0.0, wz=0.0):
    """The BLE Setpoint value: `seq u32` LE, then the CDR (72 bytes)."""
    return rp.ble_setpoint_encode(seq, {"vx": vx, "vy": vy, "wz": wz})


class Status(NamedTuple):
    """Shared state, stop reason and last refused owner ARM/STOP (NotReady, Invalid).
    Only successful owner control or release clears the error; setpoints and echo do not.
    Busy is a claimant reply, not a stored owner error."""
    state: str
    reason: str | None
    error: str | None

    @classmethod
    def decode(cls, b):
        """Raises `rovi_protocol.DecodeError` (a ValueError) for malformed bytes."""
        d = rp.status_decode(bytes(b))
        return cls(_camel(d["state"]), _camel(d["reason"]), _camel(d["error"]))


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

    LEN = rp.CAPABILITIES_LEN

    @classmethod
    def decode(cls, b):
        """Validated by the crate: version 1, finite positive geometry, finite non-negative
        limits, non-zero windows. Raises `rovi_protocol.DecodeError` otherwise."""
        return cls(**rp.capabilities_decode(bytes(b)))
