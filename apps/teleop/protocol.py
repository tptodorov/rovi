"""Version-one framing and one UDP connection attempt; no socket effects.

Frames are encoded and validated by the `rovi_protocol` binding (ADR-0017); this module keeps the
client's session rules only."""
import struct

import rovi_protocol as rp
from apps.client.wire import Capabilities, Status

HELLO, ACK, SETPOINT, ARM, STOP, BYE, STATUS, BUSY, ECHO, MODE, INFO = range(1, 12)
# ADR-0016 mode command values and network-information names (8 bytes: mode, pending, reason, 0, IPv4).
HOME, CAR_NETWORK = 1, 2
NET_MODES = ("fallback", "attempting", "home")
NET_PENDING = ("none", "home", "car-network")
NET_REASONS = ("none", "no-settings", "scan", "join", "dhcp", "router", "link-lost", "requested")


def network_text(info):
    """The car's network information as one logfmt line, as the ROS topic prints it."""
    try:
        n = rp.network_decode(bytes(info))
    except rp.DecodeError:
        return "network information not understood"
    return f"mode={n['mode']} pending={n['pending']} reason={n['reason']} ip={n['ip']}"
HEADER = struct.Struct("<2sBBII")
COMMAND = {HOME: "home", CAR_NETWORK: "car-network"}


class ClientError(Exception):
    """A fatal client error that exits through cleanup."""


class LinkError(ClientError):
    """A connection failure that permits a fresh, disarmed attempt."""


def frame(kind, session, sequence, payload=b""):
    """A raw frame, unchecked: for tests that need arbitrary or malformed input. The client sends
    only `encode` frames."""
    return HEADER.pack(b"RV", 1, kind, session, sequence) + payload


def encode(session, sequence, message):
    """A client frame, encoded and checked by the crate."""
    return rp.udp_encode_to_car({"session": session, "seq": sequence, "message": message})


def parse(data):
    """`(kind, session, sequence, payload)` of a frame whose header the crate accepts; each
    payload is then checked by its own decoder. ValueError (`rovi_protocol.DecodeError`)
    otherwise."""
    return rp.udp_split(bytes(data))


def capabilities(payload):
    try:
        caps = Capabilities.decode(payload)
    except rp.DecodeError as e:
        raise ClientError(f"invalid capabilities ({e})") from None
    if not rp.meets_client_minimum(caps._asdict()):
        raise ClientError(f"unsupported windows: need a lease of at least {rp.MIN_LEASE_MS} ms "
                          f"and a claim window of at least {rp.MIN_CLAIM_MS} ms")
    return caps


class Session:
    def __init__(self, sequence=0):
        self.session = 0
        self.caps = None
        self.sequence = sequence
        self.setpoint_sequence = 0
        self.hello = None
        self.started = False
        self.pending = None
        self.echoes = {}
        self.status = None
        self.status_at = None
        self.echo_at = None
        self.info_q = None

    def next_sequence(self):
        self.sequence = (self.sequence + 1) & 0xffffffff
        if self.sequence == 0:
            self.sequence = 1
        return self.sequence

    def start(self, now):
        if self.started:
            raise ClientError("connection attempt already started")
        self.started = True
        q = self.next_sequence()
        self.hello = (q, now + 1000)
        return encode(0, q, {"kind": "hello"})

    def setpoint(self, vector):
        self.setpoint_sequence = (self.setpoint_sequence + 1) & 0xffffffff
        vx, vy, wz = vector
        setpoint = {"vx": vx, "vy": vy, "wz": wz}
        return encode(self.session, self.setpoint_sequence, {"kind": "setpoint", "setpoint": setpoint})

    def control(self, kind, now, payload=b""):
        if not self.caps or not ((kind in (ARM, STOP) and not payload) or (kind == MODE and len(payload) == 1)):
            raise ClientError("no valid control session")
        q = self.next_sequence()
        self.pending = (kind, q, now + 500)
        message = {ARM: {"kind": "arm"}, STOP: {"kind": "stop"}}.get(kind) \
            or {"kind": "mode", "command": COMMAND.get(payload[0], "")}
        return encode(self.session, q, message)

    def info(self, now):
        """An INFO query (any peer may ask; the reply carries session 0)."""
        self.info_q = self.next_sequence()
        return encode(0, self.info_q, {"kind": "info"})

    def echo(self, now):
        q = self.next_sequence()
        nonce = struct.pack("<QI", now, q)
        self.echoes[q] = (nonce, now + 500)
        return encode(self.session, q, {"kind": "echo", "data": nonce})

    def bye(self):
        return encode(self.session, self.next_sequence(), {"kind": "bye"})

    def check(self, now):
        if self.hello and now >= self.hello[1]:
            raise LinkError("HELLO reply timeout")
        if self.pending and now >= self.pending[2]:
            raise LinkError("control reply timeout")
        if any(now >= deadline for _, deadline in self.echoes.values()):
            raise LinkError("Echo reply timeout")

    def receive(self, data, now):
        try:
            kind, session, q, payload = parse(data)
        except ValueError:
            return None
        if self.hello:
            if q != self.hello[0] or now >= self.hello[1]:
                return None
            if kind == BUSY:
                raise LinkError("car is busy")
            if kind != ACK or len(payload) != rp.CAPABILITIES_LEN:
                return None  # a truncated ACK is malformed framing, not capabilities
            if not session:
                raise ClientError("invalid zero session")
            # Retain the returned session for best-effort cleanup even if caps fail validation.
            self.session = session
            self.caps = capabilities(payload)
            self.hello = None
            return ('claimed', self.caps)
        if kind == INFO and self.info_q is not None and q == self.info_q:
            self.info_q = None
            try:
                info = rp.udp_decode_from_car(bytes(data))["message"]
            except ValueError:
                return None
            status = Status.decode(rp.status_encode(info["status"]))
            return ('info', status, rp.network_encode(info["network"]))
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
            if q == 0 and status == Status('Stopped', 'OwnerLost', None):
                # ADR-0016 takeover notice: the car now ignores this session.
                raise LinkError("another controller took the car")
            matched = control[0] if control and q == control[1] else None
            if matched or (status.state != 'Armed' and control and control[0] == ARM):
                self.pending = None
            self.status, self.status_at = status, now
            return ('status', status, matched)
        return None
