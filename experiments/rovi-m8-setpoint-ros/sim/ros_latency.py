#!/usr/bin/env python3
"""ROS 2 round-trip latency to the car (runs inside the ROS container, like ros_controller.py).

Publishes a monotonic timestamp as std_msgs/Int64 on /rovi/echo at ROVI_HZ (20) and times each
reply on /rovi/echo_reply. The car echoes the bytes of any node, with no claim and no arm; the
round trip is rclpy, rmw_zenoh, the router, the Wi-Fi hop to the car and back. Prints one line in
the format of sim/latency.py: ros sent=N count=C loss_pct=… median_ms=… p95_ms=… max_ms=…

    docker exec rovi-router bash -c 'source /opt/ros/jazzy/setup.bash; python3 /ros_latency.py'

ROVI_N (default 200) probes, ROVI_HZ, ROVI_GRAPH_WAIT_S (default 3, for the car to learn this node).
"""
import os
import statistics
import time

import rclpy
from rclpy.executors import SingleThreadedExecutor
from rclpy.node import Node
from rclpy.qos import QoSProfile, ReliabilityPolicy
from std_msgs.msg import Int64

N = int(os.environ.get("ROVI_N", "200"))
HZ = float(os.environ.get("ROVI_HZ", "20"))
WAIT = float(os.environ.get("ROVI_GRAPH_WAIT_S", "3"))


def stats(rtts_ms, sent):
    if not rtts_ms:
        return f"ros sent={sent} count=0 loss_pct=100.0"
    r = sorted(rtts_ms)
    p95 = r[min(len(r) - 1, int(0.95 * len(r)))]
    loss = 100.0 * (sent - len(r)) / sent
    return (f"ros sent={sent} count={len(r)} loss_pct={loss:.1f} "
            f"median_ms={statistics.median(r):.3f} p95_ms={p95:.3f} max_ms={r[-1]:.3f}")


def main():
    rclpy.init()
    node = Node("rovi_latency")
    qos = QoSProfile(depth=1, reliability=ReliabilityPolicy.BEST_EFFORT)
    pub = node.create_publisher(Int64, "/rovi/echo", qos)
    seen, rtts = set(), []

    def on_reply(m):
        if m.data in seen:
            return
        seen.add(m.data)
        rtts.append((time.monotonic_ns() - m.data) / 1e6)

    node.create_subscription(Int64, "/rovi/echo_reply", on_reply, qos)
    ex = SingleThreadedExecutor()
    ex.add_node(node)
    t_end = time.monotonic() + WAIT
    while time.monotonic() < t_end:  # let the graph settle
        ex.spin_once(timeout_sec=0.05)
    t0 = time.monotonic()
    for i in range(N):
        due = t0 + i / HZ
        while time.monotonic() < due:
            ex.spin_once(timeout_sec=max(0.0, min(0.01, due - time.monotonic())))
        m = Int64()
        m.data = time.monotonic_ns()
        pub.publish(m)
    t_end = time.monotonic() + 1.5  # late replies
    while time.monotonic() < t_end:
        ex.spin_once(timeout_sec=0.05)
    print(stats(rtts, N))
    rclpy.shutdown()


if __name__ == "__main__":
    main()
