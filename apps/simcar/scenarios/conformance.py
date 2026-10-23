#!/usr/bin/env python3
"""One conformance suite for every transport (add-home-network-mode, design D13): each shared rule
is written once and run over UDP, the sim car's virtual BLE and ROS 2 against one sim car in home
mode. Runs inside the ROS container with host networking (conformance.sh). Development evidence
only: no radio is modelled (ADR-0002).

Transport differences that are part of the contract, not gaps: ROS arms on its own stream
(ADR-0011); BLE claims on its first write; UDP and BLE release with BYE / disconnect, ROS with
/rovi/release."""
import os, socket, struct, threading, time, unittest

import carlog
from wire import cdr

HOST = "127.0.0.1"
PORT = int(os.environ.get("ROVI_PORT", "17790"))
LOG = os.environ["ROVI_CARLOG"]
LEASE_S = int(os.environ.get("ROVI_LEASE_MS", "600")) / 1000
STATES = {"open": 0, "claimed": 1, "armed": 2, "stopped": 3}
REASONS = {"none": 0, "stop": 1, "lease-expired": 2, "owner-lost": 3, "out-of-limits": 4}
ERRORS = {"none": 0, "busy": 1, "not-owner": 2, "not-ready": 3, "invalid": 4}
HOME, CAR = 1, 2


def observe_udp():
    """(status, network) from a non-owner UDP peer."""
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.settimeout(0.5)
    try:
        s.sendto(b"RV\x01\x0b" + struct.pack("<II", 0, 1), (HOST, PORT))
        p = s.recvfrom(256)[0][12:]
        return tuple(p[27:30]), tuple(p[30:38])
    finally:
        s.close()


class BleWatcher:
    """A second BLE central that only reads: the observer while BLE owns (Wi-Fi is off then)."""
    s = None

    @classmethod
    def observe(cls):
        if cls.s is None:
            cls.s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            cls.s.settimeout(0.5)
            assert cls._ask(b"\x01") == b"\x81\x01", "watcher not accepted"
        return tuple(cls._ask(b"\x04\x01")[2:5]), tuple(cls._ask(b"\x04\x03")[2:10])

    @classmethod
    def _ask(cls, req):
        cls.s.sendto(req, (HOST, PORT + 1))
        while True:
            d = cls.s.recvfrom(256)[0]
            if d[0] == req[0] | 0x80:
                return d


class Udp:
    name, auto_arms = "Udp", False

    def __init__(self):
        self.s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.s.settimeout(0.3)
        self.session, self.seq = 0, 0

    def _send(self, kind, payload=b""):
        self.seq += 1
        self.s.sendto(b"RV\x01" + bytes([kind]) + struct.pack("<II", self.session, self.seq) + payload, (HOST, PORT))

    def _reply(self):
        while True:
            d = self.s.recvfrom(256)[0]
            if struct.unpack_from("<I", d, 8)[0] == self.seq:
                return d

    def claim(self):
        self._send(1)
        d = self._reply()
        assert d[3] == 2, d
        self.session = struct.unpack_from("<I", d, 4)[0]

    def setpoint(self, vx):
        self._send(3, cdr(vx))

    def _control(self, kind, payload=b""):
        self._send(kind, payload)
        return tuple(self._reply()[12:15])

    def arm(self):
        return self._control(4)

    def stop(self):
        return self._control(5)

    def mode(self, value):
        return self._control(10, bytes([value]))

    def release(self):
        self._send(6)

    def close(self):
        self.s.close()


class Ble:
    name, auto_arms = "Ble", False

    def __init__(self):
        self.s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.s.settimeout(0.3)
        self.seq = 0

    def _ask(self, req):
        """The reply to `req` (op | 0x80); skips the car's unsolicited 0x87 drop notices."""
        self.s.sendto(req, (HOST, PORT + 1))
        while True:
            d = self.s.recvfrom(256)[0]
            if d[0] == req[0] | 0x80:
                return d

    def claim(self):
        assert self._ask(b"\x01") == b"\x81\x01", "not advertising"
        self.setpoint(0.0)  # the first write claims

    def setpoint(self, vx):
        self.seq += 1
        self.s.sendto(b"\x02" + struct.pack("<I", self.seq) + cdr(vx), (HOST, PORT + 1))

    def _control(self, cmd):
        self._ask(bytes([3, cmd]))
        return tuple(self._ask(b"\x04\x01")[2:5])

    def arm(self):
        return self._control(1)

    def stop(self):
        return self._control(2)

    def mode(self, value):
        return self._control(value + 2)

    def release(self):
        try:
            self._ask(b"\x06")
        except socket.timeout:
            pass  # the car already dropped this central (it lost the car)

    def close(self):
        self.s.close()


class Ros:
    name, auto_arms = "Ros", True
    node = None

    @classmethod
    def start(cls):
        import rclpy
        from geometry_msgs.msg import TwistStamped
        from rclpy.executors import SingleThreadedExecutor
        from rclpy.node import Node
        from rclpy.qos import QoSProfile, ReliabilityPolicy
        from std_msgs.msg import Bool, Empty, String, UInt8
        rclpy.init()
        cls.msg = dict(TwistStamped=TwistStamped, Bool=Bool, Empty=Empty, UInt8=UInt8)
        cls.node = Node("conformance")
        best = QoSProfile(depth=1, reliability=ReliabilityPolicy.BEST_EFFORT)
        cls.cmd = cls.node.create_publisher(TwistStamped, "/cmd_vel", best)
        cls.armp = cls.node.create_publisher(Bool, "/rovi/arm", best)
        cls.modep = cls.node.create_publisher(UInt8, "/rovi/mode", 10)
        cls.relp = cls.node.create_publisher(Empty, "/rovi/release", 10)
        cls.status = None
        cls.node.create_subscription(String, "/rovi/status", lambda m: setattr(cls, "status", m.data), 10)
        ex = SingleThreadedExecutor()
        ex.add_node(cls.node)
        threading.Thread(target=ex.spin, daemon=True).start()
        end = time.monotonic() + 10
        while cls.status is None or not cls.cmd.get_subscription_count():
            assert time.monotonic() < end, "ROS graph not ready"
            time.sleep(0.1)
        time.sleep(1)  # the car learns this node's publishers from liveliness

    def claim(self):
        self.setpoint(0.0)
        time.sleep(0.2)

    def setpoint(self, vx):
        m = self.msg["TwistStamped"]()
        m.twist.linear.x = vx
        self.cmd.publish(m)

    def _status(self):
        time.sleep(0.15)
        f = dict(kv.split("=", 1) for kv in self.status.split())
        return STATES[f["state"]], REASONS[f["reason"]], ERRORS[f["error"]]

    def arm(self):
        self.armp.publish(self.msg["Bool"](data=True))
        return self._status()

    def stop(self):
        self.armp.publish(self.msg["Bool"](data=False))
        return self._status()

    def mode(self, value):
        self.modep.publish(self.msg["UInt8"](data=value))
        return self._status()

    def release(self):
        self.relp.publish(self.msg["Empty"]())

    def close(self):
        pass


class Rules:
    """The shared rules. `T` is the transport adapter class; `observe` reads (status, network)
    without claiming, over a channel that stays up while `T` owns the car."""
    T = None
    observe = staticmethod(observe_udp)

    def setUp(self):
        self.off = os.path.getsize(LOG)
        self.c = self.T()
        self.wait_open()

    def tearDown(self):
        self.c.release()
        self.wait_open()
        self.c.close()
        if self.look()[1][0] != 2:  # a test left home mode: send it back home
            u = Udp()
            u.claim()
            u.mode(HOME)
            u.release()
            u.close()
            self.assertEqual(self.look()[1][0], 2, "did not return home")

    def look(self, timeout=4):
        """(status, network) once the car is not mid-attempt: Wi-Fi input is ignored while it
        rejoins, so a UDP observer gets no answer then."""
        end = time.monotonic() + timeout
        while True:
            try:
                seen = self.observe()
                if seen[1][0] != 1:  # not attempting
                    return seen
            except socket.timeout:
                pass
            assert time.monotonic() < end, "the car did not settle"
            time.sleep(0.1)

    def wait_open(self, timeout=8):
        end = time.monotonic() + timeout
        while self.look()[0][0] != 0:
            assert time.monotonic() < end, f"car not released: {self.look()}"
            time.sleep(0.1)

    def stream(self, vx, seconds):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            self.c.setpoint(vx)
            time.sleep(0.05)

    def moved(self):
        return any(e.get("ev") == "state" and any(e.get("wheels_pm", [0]))
                   for e in carlog.events(LOG, self.off))

    def armed_and_moving(self):
        self.c.claim()
        self.stream(0.5, 0.2)
        if not self.c.auto_arms:
            self.assertEqual(self.c.arm(), (2, 0, 0))
        self.stream(0.5, 0.3)
        self.assertTrue(self.moved(), "the armed owner drives")

    # --- rules ---

    def test_new_owner_moves_only_after_its_arm_rule(self):
        self.c.claim()
        self.stream(0.5, 0.4)
        self.assertEqual(self.moved(), self.c.auto_arms, "BLE/UDP need ARM; ROS arms on its stream")

    def test_stop_latches_until_a_fresh_arm(self):
        self.armed_and_moving()
        self.assertEqual(self.c.stop()[:2], (3, 1))
        if not self.c.auto_arms:
            self.assertEqual(self.c.arm()[2], 3, "ARM right after STOP needs a fresh setpoint")
        self.off = os.path.getsize(LOG)
        self.stream(0.5, 0.4)
        self.assertFalse(self.moved(), "the stream does not undo a STOP")
        self.assertEqual(self.c.arm()[0], 2, "an explicit ARM after fresh setpoints resumes")

    def test_lease_expiry_stops(self):
        self.armed_and_moving()
        time.sleep(LEASE_S + 0.3)
        found, _ = carlog.wait(LOG, self.off, carlog.ev("stop", reason="LeaseExpired"), 1)
        self.assertTrue(found)
        self.assertEqual(self.observe()[0][:2], (3, 2))

    def test_out_of_limits_stops(self):
        self.armed_and_moving()
        self.c.setpoint(9.0)
        found, _ = carlog.wait(LOG, self.off, carlog.ev("stop", reason="OutOfLimits"), 1)
        self.assertTrue(found)

    def test_mode_command_refused_while_armed_and_applied_on_release(self):
        self.armed_and_moving()
        self.assertEqual(self.c.mode(CAR)[2], 3, "not-ready while armed")
        self.assertEqual(self.observe()[1][1], 0, "nothing pending")
        self.c.stop()
        self.assertEqual(self.c.mode(CAR)[2], 0)
        self.assertEqual(self.observe()[1][1], 2, "car network pending, visible to observers")
        self.c.release()
        end = time.monotonic() + 3  # ROS samples reach the car through the router, a little later
        while (seen := self.look()[1][:3]) != (0, 0, 7):  # fallback, nothing pending, requested
            self.assertLess(time.monotonic(), end, seen)
            time.sleep(0.1)

    def test_unknown_mode_is_invalid(self):
        self.c.claim()
        self.assertEqual(self.c.mode(9)[2], 4)

    def test_observer_sees_the_owner_without_claiming(self):
        self.armed_and_moving()
        self.assertEqual(self.observe()[0][0], 2)
        self.stream(0.5, 0.2)
        self.assertEqual(self.observe()[0][0], 2, "observing changed nothing")

    def test_release_frees_the_car(self):
        self.c.claim()
        self.stream(0.0, 0.1)
        self.c.release()
        self.wait_open(timeout=2)


class UdpRules(Rules, unittest.TestCase):
    T = Udp


class BleRules(Rules, unittest.TestCase):
    T = Ble
    observe = staticmethod(BleWatcher.observe)


class RosRules(Rules, unittest.TestCase):
    T = Ros

    @classmethod
    def setUpClass(cls):
        Ros.start()


if __name__ == "__main__":
    result = unittest.main(verbosity=2, exit=False).result
    # The ROS spin thread is a daemon; exit without tearing rclpy down under it.
    os._exit(0 if result.wasSuccessful() else 1)
