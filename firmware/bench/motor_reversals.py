#!/usr/bin/env python3
"""Ten direct all-wheel reversals at the 70% product cap; bench image ONLY.

Run only after the separately announced flash/motor batch and Todor's go.
Stand, wheels clear, motor bank awake. This is full duty, NOT the gentle probe.
ROVI_TARGET=192.168.4.1:7777 python3 firmware/bench/motor_reversals.py --carlog <fresh-live-log>
"""
import argparse
from pathlib import Path
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "apps/simcar/scenarios"))
import carlog
import udp_scenarios as udp
from wire import Capabilities, cdr

BANNER = "BENCH ONLY motor-reversal max_drive_pm=700 markers_pm=83,-83"
LEAD_SECONDS, HOLD_SECONDS, REVERSALS = 5, 1, 10
FORWARD_PM = 1000


def status(client, command, want):
    client.send(command)
    reply = client.recv(expected_seq=client.seq)
    assert reply == (udp.STATUS, client.session, bytes(want)), reply


def stage(client, path, vx, expected, offset=None):
    initial = offset is not None
    offset = path.stat().st_size if offset is None else offset
    start = time.monotonic()
    found = False
    while time.monotonic() - start < HOLD_SECONDS:
        client.send(udp.SETPOINT, cdr(vx))
        # HELLO is idempotent and cannot ARM. It verifies the owner/radio survived.
        session = client.session
        reply = client.hello()
        assert reply and reply[:2] == (udp.ACK, session), ("owner/session lost", reply)
        text = carlog.read(path, offset)
        assert not carlog.PANIC.search(text), "panic during reversal batch"
        events = carlog.events(path, offset)
        # Every poll rereads the prefix; recompute its Claimed -> Armed order.
        found = False
        for event in events:
            assert event["ev"] not in ("boot", "stop", "released", "error", "dropped"), event
            if event["ev"] == "state":
                if initial and not found and event.get("state") == "Claimed":
                    assert event.get("wheels_pm") == [0] * 4, event
                    continue
                assert event.get("state") == "Armed", event
                assert event.get("wheels_pm") == expected, ("wrong motor demand", event, expected)
                found = True
        assert found or time.monotonic() - start < 0.75, "missing fresh motor demand"
        time.sleep(0.05)
    assert found, ("missing fresh motor demand", expected)


def cleanup(client, path):
    offset = path.stat().st_size
    try:
        for _ in range(3):
            client.send(udp.STOP)
            client.send(udp.BYE)
            found, _ = carlog.wait(path, offset, carlog.ev("released"), timeout=0.4)
            if found:
                print("STOP/BYE: serial release observed", flush=True)
                return
        raise AssertionError("cleanup release not observed")
    finally:
        client.s.close()


def check(path):
    assert udp.TARGET, "set ROVI_TARGET explicitly"
    text = carlog.read(path)
    assert BANNER in text, "bench image marker absent; refusing to claim or arm"
    problems, boots, _ = carlog.check(path)
    assert not problems and boots == 1, ("use a fresh capture with one clean boot", problems, boots)
    print(f"Full-duty bench: first motion in about {LEAD_SECONDS + HOLD_SECONDS} s", flush=True)
    time.sleep(LEAD_SECONDS)
    client = udp.Client()
    owned = False
    try:
        hello = client.hello()
        assert hello and hello[0] == udp.ACK and hello[1] != 0, hello
        owned = True
        caps = Capabilities.decode(hello[2])
        assert caps.lease_ms == 1000 and caps.max_vx == 1.0, caps
        offset = path.stat().st_size
        client.send(udp.SETPOINT, cdr())
        status(client, udp.ARM, (2, 0, 0))
        stage(client, path, 0, [0] * 4, offset)
        for index in range(REVERSALS + 1):
            direction = 1 if index % 2 == 0 else -1
            print(f"Hold {index + 1}/11: {'forward' if direction > 0 else 'reverse'} at 70%", flush=True)
            stage(client, path, direction * 0.1, [direction * FORWARD_PM] * 4)
        print("Zero/coast, then STOP and release", flush=True)
        stage(client, path, 0, [0] * 4)
        status(client, udp.STOP, (3, 1, 0))
    finally:
        if owned:
            cleanup(client, path)
        else:
            client.s.close()
    print("Ten direct reversals: serial/session checks passed; physical observation required", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--carlog", type=Path, required=True)
    check(parser.parse_args().carlog)
