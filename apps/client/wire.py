"""The car's payloads for the UDP and BLE clients: setpoint CDR, Status and Capabilities.

A thin adapter over the `rovi_protocol` binding of the rovi-protocol crate (ADR-0017): layouts,
validation and names live only there. Status fields use the crate's spelling (`stopped`,
`lease-expired`, `not-ready`, ...), with None for "none" (design D7/I4).
"""
from typing import NamedTuple

import rovi_protocol as rp

def _name(name):
    return None if name == "none" else name


def cdr(vx=0.0, vy=0.0, wz=0.0, sec=0, nanosec=0):
    """TwistStamped CDR (68 bytes) with an empty frame id."""
    return rp.setpoint_encode({"vx": vx, "vy": vy, "wz": wz, "sec": sec, "nanosec": nanosec})


def ble_setpoint(seq, vx=0.0, vy=0.0, wz=0.0):
    """The BLE Setpoint value: `seq u32` LE, then the CDR (72 bytes)."""
    return rp.ble_setpoint_encode(seq, {"vx": vx, "vy": vy, "wz": wz})


class Status(NamedTuple):
    """Shared state, stop reason and last refused owner ARM/STOP (not-ready, invalid).
    Only successful owner control or release clears the error; setpoints and echo do not.
    Busy is a claimant reply, not a stored owner error."""
    state: str
    reason: str | None
    error: str | None

    @classmethod
    def decode(cls, b):
        """Raises `rovi_protocol.DecodeError` (a ValueError) for malformed bytes."""
        d = rp.status_decode(bytes(b))
        return cls(d["state"], _name(d["reason"]), _name(d["error"]))


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
