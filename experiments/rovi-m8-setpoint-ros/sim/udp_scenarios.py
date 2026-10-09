#!/usr/bin/env python3
"""Runs the M8 UDP scenarios on real sockets, against a fresh sim car per test or an already running car.

    (cd sim/simcar && cargo build) && python3 sim/udp_scenarios.py            # development evidence only

    ROVI_TARGET=192.168.4.1:7777 python3 sim/udp_scenarios.py                 # the board: join its AP first
    ROVI_LEASE_MS, ROVI_CLAIM_MS, ROVI_RECLAIM_MS    the car's windows (board default: Config::DEFAULT)
    ROVI_CARLOG=<file>    the car's `ev=` log (sim stdout is appended; for a board, its serial capture).
                          Without it a board run has client-side checks only.
"""
import os, socket, struct, subprocess, sys, tempfile, time, unittest, pathlib

import carlog

HELLO, ACK, SETPOINT, ARM, STOP, BYE, STATUS, BUSY = 1, 2, 3, 4, 5, 6, 7, 8
TARGET = os.environ.get("ROVI_TARGET")
HOST, PORT = (TARGET.rsplit(":", 1)[0], int(TARGET.rsplit(":", 1)[1])) if TARGET else ("127.0.0.1", 17777)
# ms; the sim car gets short windows so the tests are quick
LEASE, CLAIM, RECLAIM = (int(os.environ.get(k, d)) for k, d in zip(
    ("ROVI_LEASE_MS", "ROVI_CLAIM_MS", "ROVI_RECLAIM_MS"), (1000, 10000, 5000) if TARGET else (400, 1500, 800)))
BIN = pathlib.Path(__file__).resolve().parent / "simcar/target/debug/simcar"


def ev(name, **kv):
    """Predicate for an event line with these fields."""
    return lambda e: e.get("ev") == name and all(e.get(k) == v for k, v in kv.items())


def signs(wheels):
    return tuple((w > 0) - (w < 0) for w in wheels)


def cdr(vx=0.0, vy=0.0, wz=0.0):
    """TwistStamped CDR with stamp 0, empty frame_id (NUL plus 3 pad bytes), then the twist."""
    return b"\x00\x01\x00\x00" + struct.pack("<iiI", 0, 0, 1) + b"\x00" * 4 + struct.pack("<6d", vx, vy, 0, 0, 0, wz)


class Client:
    def __init__(self):
        self.s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.s.settimeout(0.15)
        self.session, self.seq = 0, 0

    def send(self, kind, payload=b"", session=None, seq=None):
        self.seq += 1
        s = self.session if session is None else session
        q = self.seq if seq is None else seq
        self.s.sendto(b"RV\x01" + bytes([kind]) + struct.pack("<II", s, q) + payload, (HOST, PORT))

    def recv(self):
        try:
            d, _ = self.s.recvfrom(1500)
        except socket.timeout:
            return None
        assert d[:3] == b"RV\x01", d
        return d[3], struct.unpack("<II", d[4:12])[0], d[12:]

    def hello(self):
        self.send(HELLO)
        r = self.recv()
        if r and r[0] == ACK:
            self.session = r[1]
        return r

    def status(self, kind):
        self.send(kind)
        r = self.recv()
        assert r and r[0] == STATUS, r
        return tuple(r[2])  # (state, reason, error)


class Scenarios(unittest.TestCase):
    def setUp(self):
        self.a, self.b = Client(), Client()
        self.p = None
        if TARGET:  # a running car keeps its owner until the windows run out, so wait for it to be free
            self.wait_free()
            self.logpath = os.environ.get("ROVI_CARLOG")
        else:  # a fresh sim car per test, so scenarios don't depend on each other's timeouts
            log = os.environ.get("ROVI_CARLOG")
            self.log = open(log, "ab") if log else tempfile.NamedTemporaryFile()
            self.logpath = self.log.name
            self.off = os.path.getsize(self.logpath)
            self.p = subprocess.Popen([BIN, str(PORT), str(LEASE), str(CLAIM), str(RECLAIM)], stdout=self.log)
            time.sleep(0.2)
        self.off = os.path.getsize(self.logpath) if self.logpath else 0

    def tearDown(self):
        for c in (self.a, self.b):
            c.send(BYE)  # frees a running car for the next test
            c.s.close()
        if self.p:
            time.sleep(0.1)  # let the car log the release before it is stopped
            self.p.terminate()
            self.p.wait()
            self.log.close()

    def wait_free(self):
        probe = Client()
        for _ in range((CLAIM + RECLAIM + LEASE) // 100 + 20):
            r = probe.hello()
            if r and r[0] == ACK:
                probe.send(BYE)
                time.sleep(0.1)
                probe.s.close()
                return
            time.sleep(0.1)
        self.fail("the car never became free")

    def saw(self, pred):
        """The car's own log shows an event matching pred (within 1 s). Skipped when there is no car log."""
        if not self.logpath:
            return
        end = time.time() + 1
        while not any(map(pred, seen := carlog.events(self.logpath, self.off))):
            if time.time() > end:
                self.fail(f"car log lacks the event; last seen: {seen[-4:]}")
            time.sleep(0.05)

    def drive(self, c, n=1, vx=0.5):
        for _ in range(n):
            c.send(SETPOINT, cdr(vx))
            time.sleep(0.02)

    def test_claim_busy_arm_stream_stop(self):
        kind, session, caps = self.a.hello()
        self.assertEqual((kind, len(caps)), (ACK, 27))
        self.assertEqual(self.b.hello()[0], BUSY)
        self.assertEqual(self.a.status(ARM), (1, 0, 3), "arm needs a fresh setpoint")
        self.drive(self.a)
        self.assertEqual(self.a.status(ARM), (2, 0, 0))
        self.drive(self.a, 5)
        self.assertEqual(self.a.status(STOP), (3, 1, 0))
        self.saw(ev("claimed", source="Udp"))
        self.saw(ev("state", state="Armed"))
        self.saw(ev("stop", reason="Stop"))

    def test_lease_expiry_then_reclaim_without_reconnect(self):
        self.a.hello(); self.drive(self.a); self.assertEqual(self.a.status(ARM), (2, 0, 0))
        time.sleep(LEASE / 1000 + 0.1)
        self.assertEqual(self.a.status(STOP)[0], 3)  # already stopped by the lease
        self.saw(ev("stop", reason="LeaseExpired"))
        self.drive(self.a)
        self.assertEqual(self.a.status(ARM), (2, 0, 0), "same owner re-arms inside the reclaim window")

    def test_owner_released_after_reclaim_window_and_session_changes(self):
        _, s1, _ = self.a.hello(); self.drive(self.a); self.a.status(ARM)
        time.sleep((LEASE + RECLAIM) / 1000 + 0.2)
        self.assertEqual(self.b.hello()[:2], (ACK, s1 + 1))
        self.a.send(SETPOINT, cdr(0.5), session=s1)  # old session is never valid again
        self.assertIsNone(self.a.recv())

    def test_claim_window_expires_if_owner_stays_silent(self):
        self.a.hello()
        time.sleep(CLAIM / 1000 + 0.2)
        self.assertEqual(self.b.hello()[0], ACK)

    def test_bye_reopens(self):
        self.a.hello(); self.a.send(BYE); time.sleep(0.1)
        self.assertEqual(self.b.hello()[0], ACK)
        self.saw(ev("released"))

    def test_non_owner_input_is_ignored(self):
        self.a.hello(); self.drive(self.a); self.a.status(ARM)
        self.b.session = self.a.session
        self.b.send(STOP)
        self.assertIsNone(self.b.recv())
        self.assertEqual(self.a.status(ARM), (2, 0, 0), "owner still armed")

    def test_setpoints_reach_the_wheels_with_mecanum_signs(self):
        self.a.hello(); self.drive(self.a); self.assertEqual(self.a.status(ARM), (2, 0, 0))
        for (vx, vy, wz), want in [((0.5, 0, 0), (1, 1, 1, 1)), ((0, 0.5, 0), (-1, 1, -1, 1)), ((0, 0, 1), (-1, 1, 1, -1))]:
            for _ in range(10):
                self.a.send(SETPOINT, cdr(vx, vy, wz))
                time.sleep(0.02)
            self.saw(lambda e: e.get("ev") == "state" and e["state"] == "Armed" and signs(e["wheels_pm"]) == want)


if __name__ == "__main__":
    unittest.main(argv=[sys.argv[0], "-v"])
