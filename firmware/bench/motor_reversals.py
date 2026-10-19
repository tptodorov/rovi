#!/usr/bin/env python3
"""Full-duty motor bench: ten reversals, or --endurance for five minutes forward.

Run only after the separately announced flash/motor batch and Todor's go.
Stand, wheels clear, motor bank awake. This is full duty, NOT the gentle probe.
ROVI_TARGET=192.168.4.1:7777 python3 firmware/bench/motor_reversals.py --carlog <fresh-live-log>
Endurance measures running/session stability only; no temperature is measured.
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
ENDURANCE_SECONDS = 300
FORWARD_PM = 1000


def status(client, command, want):
    client.send(command)
    reply = client.recv(expected_seq=client.seq)
    assert reply == (udp.STATUS, client.session, bytes(want)), reply


def stage(client, path, vx, expected, offset=None, *, seconds=None):
    initial = offset is not None
    offset = path.stat().st_size if offset is None else offset
    start = time.monotonic()
    duration = HOLD_SECONDS if seconds is None else seconds
    assert duration > 0, "hold duration must be positive"
    # A timed endurance hold starts after fresh full-scale evidence, not delivery setup.
    hold_start = start if seconds is None else None
    next_report = 30
    found = False
    while hold_start is None or time.monotonic() - hold_start < duration:
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
        if found and hold_start is None:
            hold_start = time.monotonic()
        if seconds is not None and hold_start is not None and time.monotonic() - hold_start >= next_report:
            print(f"Hold: {next_report}/{duration} s; serial/session checks passing; temperature unmeasured", flush=True)
            next_report += 30
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


def check(path, *, endurance=False):
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
        if endurance:
            print("Five minutes forward at 70%; continuously supervise; temperature unmeasured", flush=True)
            stage(client, path, 0.1, [FORWARD_PM] * 4, seconds=ENDURANCE_SECONDS)
        else:
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
    if endurance:
        print("Five-minute endurance: serial/session checks passed; physical observation required; thermal acceptance pending", flush=True)
    else:
        print("Ten direct reversals: serial/session checks passed; physical observation required", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--carlog", type=Path, required=True)
    parser.add_argument("--endurance", action="store_true",
                        help="five minutes forward at 70%%; no thermal qualification")
    args = parser.parse_args()
    check(args.carlog, endurance=args.endurance)
