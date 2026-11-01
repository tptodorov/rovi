"""Bounded ROS bench controller; JSON UDP commands on loopback, never a public service."""
import json
import socket
import time

import rclpy
from geometry_msgs.msg import TwistStamped
from rclpy.node import Node
from rclpy.qos import QoSProfile, ReliabilityPolicy
from std_msgs.msg import Bool, Empty, String, UInt8

rclpy.init()
n = Node('rovi_home_stand')
qos = QoSProfile(depth=1, reliability=ReliabilityPolicy.BEST_EFFORT)
cmd = n.create_publisher(TwistStamped, '/cmd_vel', qos)
arm = n.create_publisher(Bool, '/rovi/arm', qos)
mode = n.create_publisher(UInt8, '/rovi/mode', 10)
release = n.create_publisher(Empty, '/rovi/release', 10)
seen = {}

def observe(key, message):
    if seen.get(key) != message.data:
        seen[key] = message.data
        print(json.dumps({'at': time.time(), key: message.data}), flush=True)

for topic in ('status', 'network'):
    n.create_subscription(String, '/rovi/' + topic, lambda m, t=topic: observe(t, m), 10)
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(('127.0.0.1', 18762))
s.setblocking(False)
stream = False
drive_until = 0
end = time.monotonic() + 600
next_send = 0
try:
    while time.monotonic() < end:
        rclpy.spin_once(n, timeout_sec=0.01)
        try:
            data, peer = s.recvfrom(1024)
        except BlockingIOError:
            pass
        else:
            request = json.loads(data)
            op = request['op']
            if op == 'stream':
                stream = True
            elif op == 'drive':
                stream = True
                drive_until = time.monotonic() + 1.0
            elif op == 'zero':
                drive_until = 0
            elif op == 'arm':
                arm.publish(Bool(data=True))
            elif op == 'stop':
                drive_until = 0
                arm.publish(Bool(data=False))
            elif op == 'release':
                stream = False
                release.publish(Empty())
            elif op == 'mode':
                mode.publish(UInt8(data=request['value']))
            elif op == 'exit':
                end = 0
            elif op != 'info':
                raise ValueError(op)
            print(json.dumps({'at': time.time(), 'command': request}), flush=True)
            s.sendto(json.dumps({'seen': seen, 'subscriptions': cmd.get_subscription_count()}).encode(), peer)
        now = time.monotonic()
        if stream and now >= next_send:
            msg = TwistStamped()
            msg.twist.linear.x = 0.1 if now < drive_until else 0.0
            cmd.publish(msg)
            next_send = now + 0.05
finally:
    if stream:
        cmd.publish(TwistStamped())
    arm.publish(Bool(data=False))
    release.publish(Empty())
    for _ in range(3):
        rclpy.spin_once(n, timeout_sec=0.05)
    s.close()
    n.destroy_node()
    rclpy.shutdown()
