#!/usr/bin/env python3
"""Announced ROS motor batch; requires Todor's go and live serial capture.

Run inside the Jazzy router container. Simulator validation uses the same probe.
"""
import os
from pathlib import Path
import time

import rclpy
from geometry_msgs.msg import TwistStamped
from rclpy.node import Node
from rclpy.qos import QoSProfile, ReliabilityPolicy
from std_msgs.msg import Bool, String

import carlog

LEAD_SECONDS = 5
DRIVE_SECONDS = 3
PAUSE_SECONDS = 1.5


def check(path):
    node = Node("rovi_stop_probe")
    qos = QoSProfile(depth=1, reliability=ReliabilityPolicy.BEST_EFFORT)
    cmd = node.create_publisher(TwistStamped, "/cmd_vel", qos)
    arm = node.create_publisher(Bool, "/rovi/arm", qos)
    received = []

    def observe(message):
        fields = dict(token.split("=", 1) for token in message.data.split())
        assert fields.keys() == {"state", "reason", "error", "session"}, fields
        received.append(fields)

    node.create_subscription(String, "/rovi/status", observe, 10)

    def spin(seconds):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            rclpy.spin_once(node, timeout_sec=min(0.02, max(0, end-time.monotonic())))

    def wait_status(start, state, reason="none", error="none", timeout=1.5):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            for fields in received[start:]:
                if (fields["state"], fields["reason"], fields["error"]) == (state, reason, error):
                    assert (fields["session"] == "0") == (state == "open"), fields
                    return fields
            spin(0.02)
        raise AssertionError((state, reason, error, received[start:]))

    def serial(offset, name, **fields):
        found, seen = carlog.wait(path, offset, carlog.ev(name, **fields), timeout=1)
        assert found, (name, fields, seen)

    def send(vx):
        message = TwistStamped()
        message.twist.linear.x = vx
        cmd.publish(message)

    def control(value):
        message = Bool()
        message.data = value
        arm.publish(message)

    def stream(seconds):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            send(0.1)
            spin(0.05)

    try:
        end = time.monotonic() + 5
        while time.monotonic() < end:
            if cmd.get_subscription_count() and arm.get_subscription_count() and received:
                break
            spin(0.05)
        assert cmd.get_subscription_count() and arm.get_subscription_count(), "ROS graph not ready"
        wait_status(0, "open")
        for mode, reason, ros_reason in (
            ("STOP", "Stop", "stop"),
            ("LEASE", "LeaseExpired", "lease-expired"),
            ("LIMITS", "OutOfLimits", "out-of-limits"),
        ):
            print(f"\n=== ROS {mode}: starts in {LEAD_SECONDS} s ===", flush=True)
            spin(LEAD_SECONDS)
            start, offset = len(received), path.stat().st_size
            send(0.1)  # A valid first ROS sample claims and auto-arms.
            wait_status(start, "armed")
            serial(offset, "claimed", source="Ros")
            serial(offset, "state", state="Armed")
            print("Gentle forward vx=0.1 for 3 s", flush=True)
            stream(DRIVE_SECONDS)
            start, offset = len(received), path.stat().st_size
            if mode == "STOP":
                control(False)
            elif mode == "LIMITS":
                print("Send vx=2.0 (limit 1.0): wheels must stop", flush=True)
                send(2.0)
            else:
                print("Stream silent: wheels must stop after about 1 s", flush=True)
            wait_status(start, "stopped", ros_reason)
            serial(offset, "stop", reason=reason)
            pause_offset = path.stat().st_size
            if mode == "LEASE":
                print("Silent pause for 1.5 s: wheels must stay still", flush=True)
                start = len(received)
                spin(PAUSE_SECONDS)
                wait_status(start, "stopped", ros_reason)
                assert not any(carlog.ev("state", state="Armed")(e)
                               for e in carlog.events(path, pause_offset)), "armed while silent"
                print("Resume commands: ROS auto-recovery for 3 s, then STOP", flush=True)
                start, offset = len(received), path.stat().st_size
                send(0.1)
            else:
                start = len(received)
                control(True)
                wait_status(start, "stopped", ros_reason, "not-ready")
                print("Fresh commands for 1.5 s without ARM: wheels must stay still", flush=True)
                start = len(received)
                stream(PAUSE_SECONDS)
                wait_status(start, "stopped", ros_reason, "not-ready")
                assert not any(f["state"] == "armed" for f in received[start:]), received[start:]
                assert not any(carlog.ev("state", state="Armed")(e)
                               for e in carlog.events(path, pause_offset)), "stream undid stop"
                print("Explicit ARM: gentle recovery for 3 s, then STOP", flush=True)
                start, offset = len(received), path.stat().st_size
                control(True)
            wait_status(start, "armed")
            serial(offset, "state", state="Armed")
            stream(DRIVE_SECONDS)
            start, offset = len(received), path.stat().st_size
            control(False)
            wait_status(start, "stopped", "stop")
            serial(offset, "stop", reason="Stop")
            start = len(received)
            wait_status(start, "open", timeout=7)
            serial(offset, "released")
            print(f"ROS {mode}: protocol/serial checks passed; wheel observation required", flush=True)
        print("ROS stop checks: passed; released", flush=True)
    finally:
        control(False)
        spin(0.1)
        node.destroy_node()


if __name__ == "__main__":
    path = Path(os.environ["ROVI_CARLOG"])
    assert path.is_file(), "live serial capture required"
    rclpy.init()
    try:
        check(path)
    finally:
        rclpy.shutdown()
