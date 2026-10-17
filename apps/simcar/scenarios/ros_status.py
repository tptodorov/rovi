#!/usr/bin/env python3
"""Jazzy status changes, heartbeat and shared UDP errors (sim only; never arms)."""
import json
import time

import rclpy
from std_msgs.msg import String

from udp_scenarios import Client, ACK, ARM, STOP, SETPOINT, BYE, ECHO, BUSY, STATUS
from wire import cdr


def check(node):
    received = []

    def observe(message):
        tokens = message.data.split()
        fields = dict(token.split('=', 1) for token in tokens)
        assert len(tokens) == 4 and fields.keys() == {'state', 'reason', 'error', 'session'}, message.data
        assert fields['state'] in {'open', 'claimed', 'armed', 'stopped'}, fields
        if fields['state'] != 'stopped':
            assert fields['reason'] == 'none', fields
        if fields['state'] == 'open':
            assert fields == dict(state='open', reason='none', error='none', session='0'), fields
        received.append((time.monotonic(), fields))

    subscription = node.create_subscription(String, '/rovi/status', observe, 10)

    def wait_for(predicate, start=0, timeout=1.0):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            for sample in received[start:]:
                if predicate(sample[1]):
                    return sample
            rclpy.spin_once(node, timeout_sec=0.01)
        raise AssertionError(received)

    # Discovery has a separate allowance; once discovered, late delivery is within one second.
    end = time.monotonic() + 5
    while not node.get_publishers_info_by_topic('/rovi/status') and time.monotonic() < end:
        rclpy.spin_once(node, timeout_sec=0.05)
    assert node.get_publishers_info_by_topic('/rovi/status')
    started = time.monotonic()
    wait_for(lambda f: f['state'] == 'open')
    late = received[-1][0] - started
    wait_for(lambda _: len(received) >= 3, timeout=2.5)
    gaps = [b[0] - a[0] for a, b in zip(received, received[1:])]
    assert max(gaps) <= 1.0, gaps

    def expect(action, state, reason='none', error='none', session=None):
        start, sent = len(received), time.monotonic()
        action()
        _, fields = wait_for(lambda f: f['state'] == state and f['reason'] == reason
            and f['error'] == error and f['session'] == str(session or 0), start, 0.3)
        return time.monotonic() - sent

    client, intruder = Client(), Client()
    try:
        start, sent = len(received), time.monotonic()
        assert client.hello()[0] == ACK
        session = client.session
        first = wait_for(lambda f: f['state'] == 'claimed' and f['session'] == str(session), start, 0.3)
        claim_delay = first[0] - sent
        expect(lambda: client.status(ARM), 'claimed', error='not-ready', session=session)
        client.send(SETPOINT, cdr(0.0))
        client.send(ECHO, b'status')
        assert client.recv(expected_seq=client.seq)[2] == b'status'
        assert intruder.hello()[0] == BUSY
        intruder.send(STOP, session=session)
        # Valid streaming, echo and non-owner control cannot clear the error.
        start = len(received)
        wait_for(lambda f: f['error'] == 'not-ready' and f['session'] == str(session), start)
        expect(lambda: client.status(STOP), 'stopped', 'stop', session=session)
        expect(lambda: client.status(ARM), 'stopped', 'stop', 'not-ready', session)
        client.send(SETPOINT, cdr(0.0))
        start = len(received)
        wait_for(lambda f: f['error'] == 'not-ready', start)
        expect(lambda: client.status(STOP), 'stopped', 'stop', session=session)
        release_delay = expect(lambda: client.send(BYE), 'open')

        # A fresh claimed session can expire without ever enabling outputs.
        start = len(received)
        assert client.hello()[0] == ACK and client.session != session
        wait_for(lambda f: f['session'] == str(client.session) and f['error'] == 'none', start, 0.3)
        client.send(SETPOINT, cdr(0.0))
        wait_for(lambda f: f['state'] == 'stopped' and f['reason'] == 'lease-expired', start, 1.3)
        assert client.recv() == (STATUS, client.session, bytes([3, 2, 0]))
        expect(lambda: client.send(BYE), 'open')
        print(json.dumps({'status_late_seconds': round(late, 3),
            'max_period_seconds': round(max(gaps), 3), 'claim_seconds': round(claim_delay, 3),
            'release_seconds': round(release_delay, 3), 'shared_errors_and_release': 'passed'}), flush=True)
    finally:
        client.send(BYE)
        client.s.close()
        intruder.s.close()
        node.destroy_subscription(subscription)


if __name__ == '__main__':
    rclpy.init()
    node = rclpy.create_node('status_conformance')
    try:
        check(node)
    finally:
        node.destroy_node()
        rclpy.shutdown()
