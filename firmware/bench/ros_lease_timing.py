#!/usr/bin/env python3
"""Announced zero-wheel ROS lease-timing batch; requires Todor's go.

Run inside the Jazzy container through ros_stops.sh ros_lease_timing.py.
Requires instrumented firmware with lease_late_ms on lease stops.
"""
import os
from pathlib import Path
import time


def checked_stop(events):
    assert not any(e.get("ev") in ("boot", "error", "dropped", "released")
                   for e in events), events
    assert all(e.get("wheels_pm") == [0, 0, 0, 0]
               for e in events if e.get("ev") == "state"), "missing/nonzero wheel command"
    stops = [(i, e) for i, e in enumerate(events) if e.get("ev") == "stop"]
    assert len(stops) <= 1, "multiple stops in one trial"
    if not stops:
        return None
    i, stop = stops[0]
    assert stop.get("reason") == "LeaseExpired", stop
    assert any(e.get("ev") == "state" and e.get("state") == "Armed"
               for e in events[:i]), "missing fresh zero-wheel Armed event before stop"
    late_ms = stop.get("lease_late_ms")
    assert isinstance(late_ms, int) and late_ms >= 0, "missing/invalid lease_late_ms"
    assert late_ms <= 50, f"device lease lateness {late_ms} ms exceeds 50 ms"
    return late_ms


def check(path):
    import carlog
    import rclpy
    from geometry_msgs.msg import TwistStamped
    from rclpy.node import Node
    from rclpy.qos import QoSProfile, ReliabilityPolicy
    from std_msgs.msg import Bool, String

    node = Node("rovi_lease_timing_probe")
    qos = QoSProfile(depth=1, reliability=ReliabilityPolicy.BEST_EFFORT)
    cmd = node.create_publisher(TwistStamped, "/cmd_vel", qos)
    arm = node.create_publisher(Bool, "/rovi/arm", qos)
    received, capabilities = [], []

    def observe(message):
        tokens = message.data.split()
        fields = dict(token.split("=", 1) for token in tokens)
        assert len(tokens) == 4 and fields.keys() == {"state", "reason", "error", "session"}
        received.append(fields)

    node.create_subscription(String, "/rovi/status", observe, 10)
    node.create_subscription(String, "/rovi/capabilities",
                             lambda m: capabilities.append(m.data), 10)

    def spin(seconds):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            rclpy.spin_once(node, timeout_sec=min(0.01, max(0, end - time.monotonic())))

    def wait_status(start, state, reason="none", session=None):
        end = time.monotonic() + 1.5
        while time.monotonic() < end:
            for i in range(start, len(received)):
                fields = received[i]
                if (fields["state"], fields["reason"], fields["error"]) == (state, reason, "none"):
                    assert (fields["session"] == "0") == (state == "open"), fields
                    assert session is None or fields["session"] == session, fields
                    return i, fields
            spin(0.01)
        raise AssertionError((state, reason, received[start:]))

    session_offset = path.stat().st_size
    results, session = [], None
    try:
        end = time.monotonic() + 5
        while not (cmd.get_subscription_count() and arm.get_subscription_count()
                   and received and capabilities):
            assert time.monotonic() < end, "ROS discovery not ready"
            spin(0.05)
        fields = dict(token.split("=", 1) for token in capabilities[-1].split())
        assert int(fields["lease_ms"]) == 1000, fields
        wait_status(0, "open")
        for trial in range(5):
            offset, start = path.stat().st_size, len(received)
            started, attempts = time.monotonic_ns(), 0
            end = time.monotonic() + 5
            # The first sample may be dropped while the car learns this publisher.
            # Stop publishing as soon as fresh serial evidence shows auto-arming.
            while True:
                cmd.publish(TwistStamped())  # All velocity fields remain zero.
                attempts += 1
                spin(0.05)
                events = carlog.events(path, offset)
                checked_stop(events)
                if any(carlog.ev("state", state="Armed")(e) for e in events):
                    break
                assert time.monotonic() < end, "ROS zero setpoint did not auto-arm"
            armed_index, armed = wait_status(start, "armed", session=session)
            session = armed["session"]
            end = time.monotonic() + 2
            while True:
                late_ms = checked_stop(carlog.events(path, offset))
                observed = time.monotonic_ns()
                if late_ms is not None:
                    break
                assert time.monotonic() < end, "missing fresh serial lease stop"
                spin(0.01)
            wait_status(armed_index + 1, "stopped", "lease-expired", session=session)
            upper_ms = (observed - started) / 1_000_000
            results.append(late_ms)
            print(f"Trial {trial + 1}/5: lease_late_ms={late_ms} limit_late_ms=50 "
                  f"host_upper_bound_ms={upper_ms:.3f} zero_publish_attempts={attempts}; "
                  "zero wheels", flush=True)
    finally:
        cleanup_offset = path.stat().st_size
        try:
            message = Bool()
            message.data = False
            arm.publish(message)
            spin(0.1)
        finally:
            node.destroy_node()
            claimed = [e for e in carlog.events(path, session_offset) if e.get("ev") == "claimed"]
            if claimed:
                found, _ = carlog.wait(path, cleanup_offset, carlog.ev("released"), timeout=8)
                assert found, "cleanup release was not observed in serial"
    assert len(claimed) == 1 and claimed[0].get("source") == "Ros", claimed
    print(f"ROS device lease timing: passed; max_lease_late_ms={max(results)}; released",
          flush=True)


if __name__ == "__main__":
    import rclpy
    path = Path(os.environ["ROVI_CARLOG"])
    assert path.is_file(), "live serial capture required"
    rclpy.init()
    try:
        check(path)
    finally:
        rclpy.shutdown()
