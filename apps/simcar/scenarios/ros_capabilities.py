#!/usr/bin/env python3
"""Jazzy late-subscriber, QoS, binary parity and claim/release checks (inside Docker)."""
import json
import os
from pathlib import Path
import time

import rclpy
from rclpy.qos import QoSProfile, ReliabilityPolicy, DurabilityPolicy
from std_msgs.msg import String

from udp_scenarios import Client, ACK, BYE
from wire import Capabilities


def check(node):
    carlog = Path(os.environ['ROVI_CARLOG'])
    offset = carlog.stat().st_size
    deadline = time.monotonic() + 5
    while not node.get_publishers_info_by_topic('/rovi/capabilities') and time.monotonic() < deadline:
        rclpy.spin_once(node, timeout_sec=0.05)
    publishers = node.get_publishers_info_by_topic('/rovi/capabilities')
    assert len(publishers) == 1, publishers
    qos = publishers[0].qos_profile
    assert qos.reliability == ReliabilityPolicy.RELIABLE, qos
    assert qos.durability == DurabilityPolicy.VOLATILE, qos
    assert qos.depth == 1, qos

    received = []
    started = time.monotonic()
    subscription = node.create_subscription(String, '/rovi/capabilities',
        lambda m: received.append((time.monotonic(), m.data)),
        QoSProfile(depth=1, reliability=ReliabilityPolicy.RELIABLE, durability=DurabilityPolicy.VOLATILE))

    def wait_for(count, timeout=1.0):
        end = time.monotonic() + timeout
        while len(received) < count and time.monotonic() < end:
            rclpy.spin_once(node, timeout_sec=0.01)
        assert len(received) >= count, (count, received)
        return received[count - 1]

    wait_for(3, 3.0)
    late = received[0][0] - started
    gaps = [b[0] - a[0] for a, b in zip(received, received[1:])]
    assert late <= 1.0, late
    assert max(gaps) <= 1.0, gaps
    tail = carlog.read_bytes()[offset:]
    assert b'ev=claimed' not in tail and b'state=Armed' not in tail, tail

    tokens = received[-1][1].split(' ')
    fields = dict(token.split('=', 1) for token in tokens)
    keys = {'version', 'wheels_radius', 'sum_of_robot_center_projection_on_X_Y_axis',
            'max_vx', 'max_vy', 'max_wz', 'lease_ms', 'claim_ms', 'reclaim_ms'}
    assert len(tokens) == 9 and fields.keys() == keys, fields
    ros = Capabilities(int(fields['version']), float(fields['wheels_radius']),
        float(fields['sum_of_robot_center_projection_on_X_Y_axis']),
        float(fields['max_vx']), float(fields['max_vy']), float(fields['max_wz']),
        int(fields['lease_ms']), int(fields['claim_ms']), int(fields['reclaim_ms']))

    # Start immediately after a periodic sample: another within 300 ms must come from the
    # ownership change, rather than the 900 ms periodic schedule.
    client = Client()
    try:
        count, claimed = len(received) + 1, time.monotonic()
        kind, session, binary = client.hello()
        assert kind == ACK and session != 0
        assert Capabilities.decode(binary) == ros, (binary, ros)
        claim_delay = wait_for(count, 0.3)[0] - claimed
        count, released = len(received) + 1, time.monotonic()
        client.send(BYE)
        release_delay = wait_for(count, 0.3)[0] - released
    finally:
        client.s.close()
    assert all(text == received[0][1] for _, text in received), received
    print(json.dumps({'late_seconds': round(late, 3), 'max_period_seconds': round(max(gaps), 3),
        'claim_seconds': round(claim_delay, 3), 'release_seconds': round(release_delay, 3),
        'capabilities': ros._asdict()}), flush=True)
    node.destroy_subscription(subscription)


if __name__ == '__main__':
    rclpy.init()
    node = rclpy.create_node('capabilities_conformance')
    try:
        check(node)
    finally:
        node.destroy_node()
        rclpy.shutdown()
