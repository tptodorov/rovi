#!/usr/bin/env python3
"""UDP and ROS share Wi-Fi: ROS cannot take the car from UDP; UDP takes it from ROS disarmed.

Announced batch; requires Todor's go and live serial capture. Run inside the Jazzy
router container (ros_stops.sh) with ROVI_TARGET set. Owners arm with zero wheels
only; the intruder streams vx=0.1, which must never reach the wheels.

1. UDP owner armed at zero, then silent. A ROS node streams vx=0.1: no ROS claim,
   wheels stay zero, AP stays on, and the UDP lease still expires ~1 s after its
   last setpoint. ROS stops; BYE releases.
2. ROS owner auto-armed at zero, then silent. UDP HELLO takes the car over (ADR-0016): an
   owner-lost stop for ROS, and the UDP owner starts disarmed, so its vx=0.1 setpoints without
   ARM never reach the wheels. UDP BYE releases. (Until 2026-10-23 phase 2 expected BUSY; see
   the 2026-10-19 bench notes for that rule's evidence.)
"""
import os
from pathlib import Path
import time

import rclpy
from geometry_msgs.msg import TwistStamped
from rclpy.node import Node
from rclpy.qos import QoSProfile, ReliabilityPolicy
from std_msgs.msg import Bool

import carlog
import udp_scenarios as udp
from wire import cdr

INTRUDE_SECONDS = 2.0


def check(path):
    node = Node("rovi_sharing_probe")
    qos = QoSProfile(depth=1, reliability=ReliabilityPolicy.BEST_EFFORT)
    cmd = node.create_publisher(TwistStamped, "/cmd_vel", qos)
    arm = node.create_publisher(Bool, "/rovi/arm", qos)

    def spin(seconds):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            rclpy.spin_once(node, timeout_sec=0.02)

    def ros_send(vx):
        message = TwistStamped()
        message.twist.linear.x = vx
        cmd.publish(message)

    def serial(offset, name, timeout=2, **fields):
        found, seen = carlog.wait(path, offset, carlog.ev(name, **fields), timeout=timeout)
        assert found, (name, fields, [e.get("ev") for e in seen[-6:]])

    def first(offset, name, **fields):
        return next(e for e in carlog.events(path, offset) if carlog.ev(name, **fields)(e))

    def never(offset, what):
        moving = [e for e in carlog.events(path, offset)
                  if e.get("ev") == "state" and any(e.get("wheels_pm", [0]))]
        claims = [e for e in carlog.events(path, offset) if e.get("ev") == "claimed"]
        ap_off = [e for e in carlog.events(path, offset) if carlog.ev("wifi", ap="off")(e)]
        assert not (moving or claims or ap_off), (what, moving, claims, ap_off)

    end = time.monotonic() + 5
    while not (cmd.get_subscription_count() and arm.get_subscription_count()):
        assert time.monotonic() < end, "ROS graph not ready"
        spin(0.05)

    print("=== 1. UDP owner, ROS intruder ===", flush=True)
    client = udp.Client()
    try:
        offset = path.stat().st_size
        assert client.hello()[0] == udp.ACK
        client.send(udp.SETPOINT, cdr())
        assert client.status(udp.ARM) == (2, 0, 0), "UDP ARM refused"
        serial(offset, "claimed", source="Udp")
        serial(offset, "state", state="Armed", wheels_pm=[0, 0, 0, 0])
        client.send(udp.SETPOINT, cdr())
        silent = path.stat().st_size
        print("UDP silent; ROS streams vx=0.1 for 2 s: wheels must stay still", flush=True)
        end = time.monotonic() + INTRUDE_SECONDS
        while time.monotonic() < end:
            ros_send(0.1)
            spin(0.05)
        serial(silent, "stop", reason="LeaseExpired")
        stop = first(silent, "stop", reason="LeaseExpired")
        # ROS input must not have renewed the UDP lease (~1 s, so stop well before 2 s).
        armed = first(offset, "state", state="Armed")
        assert stop["at_ms"] - armed["at_ms"] < 1600, (armed, stop)
        never(silent, "ROS intruder")
        print(f"UDP lease expired {stop['at_ms'] - armed['at_ms']} ms after arming; "
              "no ROS claim, zero wheels, AP on", flush=True)
        spin(0.5)
    finally:
        offset = path.stat().st_size
        client.send(udp.BYE)
        client.s.close()
    serial(offset, "released")

    print("=== 2. ROS owner, UDP takes over ===", flush=True)
    offset = path.stat().st_size
    # Zeros claim and auto-arm; the first may be dropped while the car learns the publisher.
    end = time.monotonic() + 5
    while not any(carlog.ev("state", state="Armed")(e) for e in carlog.events(path, offset)):
        assert time.monotonic() < end, "ROS zero stream did not arm"
        ros_send(0.0)
        spin(0.05)
    serial(offset, "claimed", source="Ros")
    serial(offset, "state", state="Armed", wheels_pm=[0, 0, 0, 0])
    silent = path.stat().st_size
    taker = udp.Client()
    try:
        reply = taker.hello()
        assert reply and reply[0] == udp.ACK, reply
        serial(silent, "stop", reason="OwnerLost")
        serial(silent, "claimed", source="Udp")
        after = path.stat().st_size
        print("UDP took over disarmed; it streams vx=0.1 without ARM for 2 s: wheels must stay still",
              flush=True)
        end = time.monotonic() + INTRUDE_SECONDS
        while time.monotonic() < end:
            taker.send(udp.SETPOINT, cdr(0.1))
            time.sleep(0.05)
        never(after, "disarmed UDP owner")
        offset = path.stat().st_size
        taker.send(udp.BYE)
    finally:
        taker.s.close()
    serial(offset, "released")
    node.destroy_publisher(cmd)
    node.destroy_node()
    print("UDP/ROS sharing: passed; released", flush=True)


if __name__ == "__main__":
    path = Path(os.environ["ROVI_CARLOG"])
    assert path.is_file(), "live serial capture required"
    assert os.environ.get("ROVI_TARGET"), "set ROVI_TARGET explicitly"
    rclpy.init()
    try:
        check(path)
    finally:
        rclpy.shutdown()
