#!/usr/bin/env python3
"""Runs inside the ROS 2 container: one controller node with two publishers (/cmd_vel and
/rovi/arm) plus an intruder node, against the sim car. Prints the car's /rovi/status over time.

Timeline: the stream claims and arms the car on its own (ADR-0011); at 2 s an intruder node tries
to drive and stop; at 3 s the controller stops, and the stop stays latched while it keeps
streaming; at 4.5 s it arms again; at 5.5 s the stream ends and the lease expires.
At 7 s an out-of-limits sample latches the stop; valid streaming resumes at 7.25 s,
but only the explicit arm at 9 s may restart the car."""
import os
import time
import rclpy
from geometry_msgs.msg import TwistStamped
from rclpy.executors import SingleThreadedExecutor
from rclpy.node import Node
from rclpy.qos import QoSProfile, ReliabilityPolicy
from std_msgs.msg import Bool, String

rclpy.init()
ctl, bad = Node("ctl"), Node("intruder")
qos = QoSProfile(depth=1, reliability=ReliabilityPolicy.BEST_EFFORT)
cmd, arm = ctl.create_publisher(TwistStamped, "/cmd_vel", qos), ctl.create_publisher(Bool, "/rovi/arm", qos)
bad_cmd, bad_arm = bad.create_publisher(TwistStamped, "/cmd_vel", qos), bad.create_publisher(Bool, "/rovi/arm", qos)
status = []
ctl.create_subscription(String, "/rovi/status", lambda m: status.append((round(time.time() - t0, 1), m.data)), 10)
ex = SingleThreadedExecutor()
ex.add_node(ctl)
ex.add_node(bad)


def twist(vx):
    m = TwistStamped()
    m.twist.linear.x = vx
    return m


def send_arm(pub, v):
    m = Bool()
    m.data = v
    pub.publish(m)


t0 = time.time()
time.sleep(float(os.environ.get("ROVI_GRAPH_WAIT_S", "2")))  # let the car learn the graph from liveliness tokens
t0 = time.time()
armed_sent = stopped_sent = intruder_sent = oversized_sent = rearmed_sent = False
while (t := time.time() - t0) < 11.5:
    if t < 5.5 or 7.25 < t < 10:
        cmd.publish(twist(1.0))  # 20 Hz
    if t > 2 and not intruder_sent:  # another node tries to drive and to stop
        bad_cmd.publish(twist(-1.0)); send_arm(bad_arm, False); intruder_sent = True
    if t > 3 and not stopped_sent:
        send_arm(arm, False); stopped_sent = True
    if t > 4.5 and not armed_sent:
        send_arm(arm, True); armed_sent = True
    if t > 7 and not oversized_sent:
        cmd.publish(twist(9.0)); oversized_sent = True
    if t > 9 and not rearmed_sent:
        send_arm(arm, True); rearmed_sent = True
    ex.spin_once(timeout_sec=0.05)
for t, s in status:
    print(f"{t:4} {s}")
