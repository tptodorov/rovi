#!/usr/bin/env python3
"""Network-mode and protocol-alignment scenarios (ADR-0016) over UDP and the sim car's virtual BLE.

    (cd apps/simcar && cargo build) && (cd apps/simcar/scenarios && python3 network_scenarios.py)

A fresh sim car per test; home mode is scripted through its stdin (`net.rs`), BLE goes through
the virtual BLE socket (`vble.rs`). Development evidence only: no radio is modelled (ADR-0002).
ROS cases run in the ROS scenario container.
"""
import os, socket, struct, subprocess, tempfile, time, unittest

import carlog
from carlog import ev
from udp_scenarios import BIN, Client as UdpClient, SETPOINT, ARM, STOP, BYE, STATUS, ACK, BUSY
from wire import cdr

MODE, INFO = 10, 11
HOME, CAR = 1, 2
HOST, PORT = "127.0.0.1", 17877
LEASE, CLAIM, RECLAIM = 400, 1500, 800


class Udp(UdpClient):
    def __init__(self):
        super().__init__()

    def send(self, kind, payload=b"", session=None, seq=None):
        self.seq += 1
        s = self.session if session is None else session
        q = self.seq if seq is None else seq
        self.s.sendto(b"RV\x01" + bytes([kind]) + struct.pack("<II", s, q) + payload, (HOST, PORT))

    def recv(self, expected_seq=None, timeout=0.15):
        self.s.settimeout(timeout)
        while True:
            try:
                d, _ = self.s.recvfrom(1500)
            except socket.timeout:
                return None
            kind, session, seq = d[3], *struct.unpack_from("<II", d, 4)
            if expected_seq is None or seq == expected_seq:
                return kind, session, d[12:]

    def armed(self):
        self.hello()
        self.send(SETPOINT, cdr(0.5))
        time.sleep(0.02)
        assert self.status(ARM) == (2, 0, 0)

    def mode(self, value):
        self.send(MODE, bytes([value]))
        r = self.recv(expected_seq=self.seq)
        assert r and r[0] == STATUS, r
        return tuple(r[2])

    def info(self):
        self.send(INFO, session=0)
        r = self.recv(expected_seq=self.seq)
        assert r and r[0] == INFO, r
        p = r[2]
        return p[:27], tuple(p[27:30]), tuple(p[30:38])


class Ble:
    """One virtual BLE central (see apps/simcar/src/vble.rs)."""

    def __init__(self):
        self.s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.s.bind(("", 0))
        self.seq = 0

    def _ask(self, req, reply_op, timeout=0.3):
        self.s.sendto(req, (HOST, PORT + 1))
        self.s.settimeout(timeout)
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            try:
                d, _ = self.s.recvfrom(512)
            except socket.timeout:
                return None
            if d[0] == reply_op:
                return d[1:]
        return None

    def connect(self):
        return self._ask(b"\x01", 0x81) == b"\x01"

    def setpoint(self, vx=0.5):
        self.seq += 1
        self.s.sendto(b"\x02" + struct.pack("<I", self.seq) + cdr(vx), (HOST, PORT + 1))
        time.sleep(0.02)

    def control(self, cmd):
        """0 accepted, 0x13 rejected."""
        return self._ask(bytes([3, cmd]), 0x83)[0]

    def read(self, char):
        r = self._ask(bytes([4, char]), 0x84)
        return tuple(r[1:])

    def disconnect(self):
        self._ask(b"\x06", 0x86)

    def dropped(self, timeout=1.0):
        """The car disconnected us."""
        self.s.settimeout(timeout)
        try:
            return self.s.recvfrom(512)[0] == b"\x87"
        except socket.timeout:
            return False


class NetworkScenarios(unittest.TestCase):
    HOME = True

    def setUp(self):
        self.log = tempfile.NamedTemporaryFile()
        env = dict(os.environ, ROVI_SIM_HOME="1" if self.HOME else "0")
        self.p = subprocess.Popen(
            [BIN, str(PORT), str(LEASE), str(CLAIM), str(RECLAIM)],
            stdout=self.log, stdin=subprocess.PIPE, env=env)
        time.sleep(0.3)
        self.a, self.b = Udp(), Udp()
        self.bles = []

    def tearDown(self):
        for c in self.bles:
            c.s.close()
        for c in (self.a, self.b):
            c.s.close()
        self.p.stdin.close()
        self.p.terminate()
        self.p.wait()
        self.log.close()

    def ble(self):
        c = Ble()
        self.bles.append(c)
        return c

    def script(self, line):
        self.p.stdin.write(line.encode() + b"\n")
        self.p.stdin.flush()

    def saw(self, pred, timeout=1.5):
        found, seen = carlog.wait(self.log.name, 0, pred, timeout)
        if not found:
            self.fail(f"car log lacks the event; last seen: {seen[-6:]}")

    def mode_is(self, mode, timeout=1.5, **kv):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            nets = [e for e in carlog.events(self.log.name) if e["ev"] == "net"]
            if nets and nets[-1].get("mode") == mode and all(nets[-1].get(k) == v for k, v in kv.items()):
                return
            time.sleep(0.05)
        self.fail(f"last net event is not {mode} {kv}: {nets[-1:] if nets else None}")


class HomeMode(NetworkScenarios):
    def test_boot_enters_home_and_info_reports_it(self):
        self.mode_is("Home")
        caps, status, net = self.b.info()
        self.assertEqual(len(caps), 27)
        self.assertEqual(status, (0, 0, 0))
        self.assertEqual(net, (2, 0, 0, 0, 127, 0, 0, 1))

    def test_info_from_a_non_owner_leaves_the_owner_alone(self):
        self.a.armed()
        _, status, _ = self.b.info()
        self.assertEqual(status[0], 2, "armed")
        self.assertEqual(self.a.status(STOP), (3, 1, 0), "a still owns the car")

    def test_ble_watches_without_claiming(self):
        self.a.armed()
        w = self.ble()
        self.assertTrue(w.connect(), "BLE advertises in home mode while UDP owns")
        self.assertEqual(w.read(1), (2, 0, 0))
        self.assertEqual(w.read(3)[0], 2, "home")
        self.assertEqual(self.a.status(STOP), (3, 1, 0), "UDP still owns")

    def test_ble_takes_over_udp_and_the_udp_owner_is_told(self):
        self.a.armed()
        old = self.a.session
        c = self.ble()
        self.assertTrue(c.connect())
        c.setpoint(0.5)  # the first write claims
        r = self.a.recv(timeout=0.5)
        self.assertIsNotNone(r, "takeover notice")
        self.assertEqual((r[0], r[1], tuple(r[2])), (STATUS, old, (3, 3, 0)))
        self.saw(ev("preempted", source="Udp"))
        self.saw(ev("claimed", source="Ble"))
        self.assertIsNone(self.b.hello(), "Wi-Fi is off while BLE owns")
        self.assertEqual(c.read(1)[0], 1, "BLE owner starts disarmed: a setpoint alone does not arm")

    def test_second_ble_central_cannot_claim(self):
        c1, c2 = self.ble(), self.ble()
        self.assertTrue(c1.connect() and c2.connect())
        self.assertEqual(c1.control(2), 0)
        c2.setpoint()
        self.assertEqual(c2.control(1), 0x13)
        self.assertEqual(c1.read(1)[0], 3, "c1 still owns, stopped")

    def test_ble_end_reattempts_home(self):
        c = self.ble()
        c.connect()
        c.control(2)
        c.disconnect()
        self.saw(ev("net", mode="Attempting"))
        self.mode_is("Home")

    def test_car_network_then_home_from_fallback_over_udp(self):
        c = self.ble()
        c.connect()
        c.control(2)
        self.assertEqual(c.control(4), 0)
        self.assertEqual(c.read(3)[1], 2, "pending car network is visible")
        c.disconnect()
        self.mode_is("Fallback", reason="Requested")
        self.a.hello()
        self.assertEqual(self.a.mode(HOME), (1, 0, 0))
        self.a.send(BYE)
        self.mode_is("Home")
        self.saw(ev("mode_cmd", source="Udp", cmd="Home", result="Ok"))

    def test_failed_attempt_falls_back_with_its_step(self):
        self.mode_is("Home")
        self.script("fail join")
        c = self.ble()
        c.connect()
        c.control(2)
        c.disconnect()
        self.mode_is("Fallback", reason="Join")
        self.saw(ev("net_step", step="Join", result="Failed"))

    def test_link_loss_that_cannot_rejoin_falls_back(self):
        self.mode_is("Home")
        self.script("fail scan")
        self.script("lost")
        self.mode_is("Fallback", reason="LinkLost")

    def test_mode_command_while_armed_is_refused(self):
        self.a.armed()
        self.assertEqual(self.a.mode(CAR), (2, 0, 3), "not-ready")
        self.saw(ev("mode_cmd", source="Udp", cmd="CarNetwork", result="NotReady"))

    def test_owner_released_by_its_window_is_disconnected(self):
        c = self.ble()
        c.connect()
        c.control(2)  # claims; no setpoint, so the claim window ends it
        self.assertTrue(c.dropped(timeout=(CLAIM + 500) / 1000))
        self.mode_is("Home")


class FallbackMode(NetworkScenarios):
    HOME = False

    def test_no_settings_is_todays_behaviour(self):
        self.mode_is("Fallback", reason="NoSettings")
        self.a.hello()
        self.assertEqual(self.a.mode(HOME), (1, 0, 4), "Home without settings is invalid")

    def test_ble_does_not_advertise_while_udp_owns(self):
        self.a.armed()
        self.assertFalse(self.ble().connect())

    def test_ble_owner_turns_wifi_off_and_back_on(self):
        c = self.ble()
        self.assertTrue(c.connect())
        c.control(2)
        self.assertIsNone(self.a.hello())
        c.disconnect()
        time.sleep(0.05)
        self.assertEqual(self.a.hello()[0], ACK)


if __name__ == "__main__":
    unittest.main()
