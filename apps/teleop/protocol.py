"""Version-one framing and one non-reconnecting UDP session; no socket effects."""
import math
import struct
from apps.client.wire import Capabilities, Status, cdr

HELLO, ACK, SETPOINT, ARM, STOP, BYE, STATUS, BUSY, ECHO = range(1, 10)
HEADER = struct.Struct("<2sBBII")

class ClientError(Exception):
    """The operator must start a new session after a failed run."""


def frame(kind, session, sequence, payload=b""):
    return HEADER.pack(b"RV", 1, kind, session, sequence) + payload


def parse(data):
    if len(data) < HEADER.size:
        raise ValueError("short header")
    magic, version, kind, session, sequence = HEADER.unpack_from(data)
    payload = data[HEADER.size:]
    lengths = {HELLO: 0, ACK: 27, SETPOINT: 68, ARM: 0, STOP: 0, BYE: 0, STATUS: 3, BUSY: 0}
    if magic != b"RV" or version != 1 or kind not in range(1, 10):
        raise ValueError("unknown frame")
    if (kind == ECHO and len(payload) > 32) or (kind != ECHO and len(payload) != lengths[kind]):
        raise ValueError("invalid payload length")
    return kind, session, sequence, payload


def capabilities(payload):
    caps = Capabilities.decode(payload)
    geometry = (caps.wheels_radius, caps.center_projection_sum)
    limits = (caps.max_vx, caps.max_vy, caps.max_wz)
    if (caps.version != 1 or not all(math.isfinite(x) and x > 0 for x in geometry)
            or not all(math.isfinite(x) and x >= 0 for x in limits)
            or not all(x > 0 for x in (caps.claim_ms, caps.reclaim_ms))):
        raise ClientError("invalid capabilities")
    if caps.lease_ms < 200:
        raise ClientError("unsupported lease: need at least 200 ms")
    return caps


class Session:
    def __init__(self):
        self.session = 0
        self.caps = None
        self.sequence = 0
        self.setpoint_sequence = 0
        self.hello = None
        self.started = False
        self.pending = None
        self.echoes = {}
        self.status = None
        self.status_at = None
        self.echo_at = None

    def next_sequence(self):
        self.sequence = (self.sequence + 1) & 0xffffffff
        if self.sequence == 0:
            self.sequence = 1
        return self.sequence

    def start(self, now):
        if self.started:
            raise ClientError("automatic reconnect is disabled")
        self.started = True
        q = self.next_sequence()
        self.hello = (q, now + 1000)
        return frame(HELLO, 0, q)

    def setpoint(self, vector):
        self.setpoint_sequence = (self.setpoint_sequence + 1) & 0xffffffff
        return frame(SETPOINT, self.session, self.setpoint_sequence, cdr(*vector))

    def control(self, kind, now):
        if kind not in (ARM, STOP) or not self.caps:
            raise ClientError("no valid control session")
        q = self.next_sequence()
        self.pending = (kind, q, now + 500)
        return frame(kind, self.session, q)

    def echo(self, now):
        q = self.next_sequence()
        nonce = struct.pack("<QI", now, q)
        self.echoes[q] = (nonce, now + 500)
        return frame(ECHO, self.session, q, nonce)

    def bye(self):
        return frame(BYE, self.session, self.next_sequence())

    def check(self, now):
        if self.hello and now >= self.hello[1]:
            raise ClientError("HELLO reply timeout")
        if self.pending and now >= self.pending[2]:
            raise ClientError("control reply timeout")
        if any(now >= deadline for _, deadline in self.echoes.values()):
            raise ClientError("Echo reply timeout")

    def receive(self, data, now):
        try:
            kind, session, q, payload = parse(data)
        except ValueError:
            return None
        if self.hello:
            if q != self.hello[0] or now >= self.hello[1]:
                return None
            if kind == BUSY:
                raise ClientError("car is busy")
            if kind != ACK:
                return None
            if not session:
                raise ClientError("invalid zero session")
            # Retain the returned session for best-effort cleanup even if caps fail validation.
            self.session = session
            self.caps = capabilities(payload)
            self.hello = None
            return ('claimed', self.caps)
        if not self.caps or session != self.session:
            return None
        if kind == ECHO:
            expected = self.echoes.get(q)
            if expected and now < expected[1] and payload == expected[0]:
                del self.echoes[q]
                self.echo_at = now
                return ('echo',)
        if kind == STATUS:
            control = self.pending
            if q != 0 and (not control or q != control[1] or now >= control[2]):
                return None
            try:
                status = Status.decode(payload)
            except ValueError:
                return None
            if ((status.state == "Stopped") != (status.reason is not None)
                    or status.error not in (None, "NotReady", "Invalid")):
                return None
            matched = control[0] if control and q == control[1] else None
            if matched or (status.state != 'Armed' and control and control[0] == ARM):
                self.pending = None
            self.status, self.status_at = status, now
            return ('status', status, matched)
        return None
