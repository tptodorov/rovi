#!/usr/bin/env python3
"""Runs the M8 UDP scenarios against the sim car on real sockets (development evidence only).

    (cd sim/simcar && cargo build) && python3 sim/udp_scenarios.py
"""
import socket, struct, subprocess, sys, time, unittest, pathlib

PORT = 17777
HELLO, ACK, SETPOINT, ARM, STOP, BYE, STATUS, BUSY = 1, 2, 3, 4, 5, 6, 7, 8
LEASE, CLAIM, RECLAIM = 400, 1500, 800  # ms, short so the tests are quick
BIN = pathlib.Path(__file__).resolve().parent.parent / "sim/simcar/target/debug/simcar"


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
        self.s.sendto(b"RV\x01" + bytes([kind]) + struct.pack("<II", s, q) + payload, ("127.0.0.1", PORT))

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
    def setUp(self):  # a fresh sim car per test, so scenarios don't depend on each other's timeouts
        self.p = subprocess.Popen([BIN, str(PORT), str(LEASE), str(CLAIM), str(RECLAIM)], stdout=subprocess.DEVNULL)
        time.sleep(0.2)
        self.a, self.b = Client(), Client()

    def tearDown(self):
        self.p.terminate()
        self.p.wait()
        self.a.s.close()
        self.b.s.close()

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

    def test_lease_expiry_then_reclaim_without_reconnect(self):
        self.a.hello(); self.drive(self.a); self.assertEqual(self.a.status(ARM), (2, 0, 0))
        time.sleep(LEASE / 1000 + 0.1)
        self.assertEqual(self.a.status(STOP)[0], 3)  # already stopped by the lease
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

    def test_non_owner_input_is_ignored(self):
        self.a.hello(); self.drive(self.a); self.a.status(ARM)
        self.b.session = self.a.session
        self.b.send(STOP)
        self.assertIsNone(self.b.recv())
        self.assertEqual(self.a.status(ARM), (2, 0, 0), "owner still armed")


if __name__ == "__main__":
    unittest.main(argv=[sys.argv[0], "-v"])
