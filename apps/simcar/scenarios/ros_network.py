#!/usr/bin/env python3
"""Runs inside the ROS 2 container for ros-network.sh: one controller node `ctl` streams /cmd_vel
while the host takes the car over with UDP or virtual BLE. Prints /rovi/status and /rovi/network.

ROVI_PLAN=A (fallback car): stream 0-8 s; the host's UDP takes over at 3 s and leaves without
Home at 4 s, so the stream claims again but stays latched until `/rovi/arm` true at 6 s. At 8 s
the node stops, at 8.2 s sends Car network on /rovi/mode and at 8.4 s releases.
ROVI_PLAN=B (home car): stream 0-10 s, no arm at all; the host's BLE hands back with Home at 3 s
(the stream re-arms on its own) and drops without Home at 6 s (the stream stays latched)."""
import os
import time
import rclpy
from geometry_msgs.msg import TwistStamped
from rclpy.executors import SingleThreadedExecutor
from rclpy.node import Node
from rclpy.qos import QoSProfile, ReliabilityPolicy
from std_msgs.msg import Bool, Empty, String, UInt8

PLAN = os.environ.get("ROVI_PLAN", "A")
rclpy.init()
ctl = Node("ctl")
best = QoSProfile(depth=1, reliability=ReliabilityPolicy.BEST_EFFORT)
cmd = ctl.create_publisher(TwistStamped, "/cmd_vel", best)
arm = ctl.create_publisher(Bool, "/rovi/arm", best)
mode = ctl.create_publisher(UInt8, "/rovi/mode", 10)
release = ctl.create_publisher(Empty, "/rovi/release", 10)
seen = []
ctl.create_subscription(String, "/rovi/status", lambda m: seen.append(("status", round(time.time() - t0, 1), m.data)), 10)
ctl.create_subscription(String, "/rovi/network", lambda m: seen.append(("network", round(time.time() - t0, 1), m.data)), 10)
ex = SingleThreadedExecutor()
ex.add_node(ctl)


def twist(vx):
    m = TwistStamped()
    m.twist.linear.x = vx
    return m


def once(name, cond, action):
    if cond and name not in done:
        action()
        done.add(name)


t0 = time.time()
time.sleep(float(os.environ.get("ROVI_GRAPH_WAIT_S", "2")))  # let the car learn the graph
t0 = time.time()
done = set()
end = 11 if PLAN == "A" else 10.5
while (t := time.time() - t0) < end:
    streaming = t < 8 if PLAN == "A" else t < 10
    if streaming:
        cmd.publish(twist(1.0))  # 20 Hz
    if PLAN == "A":
        once("arm", t > 6, lambda: arm.publish(Bool(data=True)))
        once("stop", t > 8, lambda: arm.publish(Bool(data=False)))
        once("mode", t > 8.2, lambda: mode.publish(UInt8(data=2)))
        once("release", t > 8.4, lambda: release.publish(Empty()))
    ex.spin_once(timeout_sec=0.05)
for kind, t, s in seen:
    print(f"{kind} {t:4} {s}")
