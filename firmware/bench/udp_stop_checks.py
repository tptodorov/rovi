#!/usr/bin/env python3
"""Visible UDP board checks; run only after an announced test and Todor's go.

Stand, wheels clear, motor bank awake; join Rovi's AP and capture serial first:
    ROVI_TARGET=192.168.4.1:7777 python3 firmware/bench/udp_stop_checks.py --carlog <live-car.log>
The target is explicit so importing this script never connects to a board.
"""
import argparse
from pathlib import Path
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "apps/simcar/scenarios"))
import carlog
import udp_scenarios as udp
from wire import Capabilities, cdr

LEAD_SECONDS, DRIVE_SECONDS, PAUSE_SECONDS = 5, 3, 1.5
MODES = ("STOP", "LEASE", "BYE", "LIMITS")


def close(client):
    try:
        client.send(udp.STOP)
    finally:
        try:
            client.send(udp.BYE)
        finally:
            client.s.close()


def stream(client, seconds):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        client.send(udp.SETPOINT, cdr(0.1))
        time.sleep(0.05)


def expect(client, command, status):
    actual = client.status(command)
    assert actual == status, (actual, status)
    print(f"Status reply: {actual}", flush=True)


def stop_event(path, offset, reason):
    found, events = carlog.wait(path, offset, carlog.ev("stop"))
    stop = next((event for event in events if event.get("ev") == "stop"), None)
    assert found and stop["reason"] == reason, (reason, stop)
    print(f"Serial stop reason={reason}", flush=True)


def notification(client, reason):
    end, reply = time.monotonic() + 1.5, None
    while reply is None and time.monotonic() < end:
        reply = client.recv()
    assert reply == (udp.STATUS, client.session, bytes([3, reason, 0])), reply
    print(f"Unsolicited owner Status: {list(reply[2])}", flush=True)


def released(path, offset):
    found, _ = carlog.wait(path, offset, carlog.ev("released"))
    assert found, "cleanup release was not observed in serial"


def recover(client, path, reason):
    expect(client, udp.ARM, (3, reason, 3))
    offset = path.stat().st_size
    print("Fresh gentle stream: wheels must stay stopped", flush=True)
    stream(client, PAUSE_SECONDS)
    assert not any(carlog.ev("state", state="Armed")(event)
                   for event in carlog.events(path, offset)), "stream re-armed without ARM"
    print("Explicit ARM: gentle forward for 3 s, then STOP", flush=True)
    expect(client, udp.ARM, (2, 0, 0))
    stream(client, DRIVE_SECONDS)
    expect(client, udp.STOP, (3, 1, 0))


def scenario(mode, path):
    assert mode in MODES
    print(f"\n=== UDP {mode}: starts in 5 s ===", flush=True)
    time.sleep(LEAD_SECONDS)
    client = udp.Client()
    try:
        hello = client.hello()
        assert hello and hello[0] == udp.ACK and hello[1] != 0, hello
        caps = Capabilities.decode(hello[2])
        assert caps.max_vx == 1.0 and caps.lease_ms == 1000, caps
        if mode == "STOP":
            intruder = udp.Client()
            try:
                assert intruder.hello()[0] == udp.BUSY
                intruder.send(udp.STOP, session=client.session)
                assert intruder.recv() is None, "non-owner STOP received a reply"
            finally:
                close(intruder)
            print("Other UDP endpoint gets BUSY; its STOP is ignored", flush=True)
        expect(client, udp.ARM, (1, 0, 3))
        client.send(udp.SETPOINT, cdr(0.1))
        expect(client, udp.ARM, (2, 0, 0))
        print("Gentle forward vx=0.1 for 3 s", flush=True)
        stream(client, DRIVE_SECONDS)
        offset = path.stat().st_size
        if mode == "STOP":
            expect(client, udp.STOP, (3, 1, 0))
            stop_event(path, offset, "Stop")
            recover(client, path, 1)
        elif mode == "LEASE":
            print("Laptop silent: wheels must stop after about 1 s", flush=True)
            notification(client, 2)  # receive only; no control request
            stop_event(path, offset, "LeaseExpired")
            recover(client, path, 2)
        elif mode == "BYE":
            print("BYE while driving: wheels must stop", flush=True)
            client.send(udp.BYE)
            stop_event(path, offset, "OwnerLost")
            again = udp.Client()
            try:
                claimed = again.hello()
                assert claimed and claimed[0] == udp.ACK, claimed
                assert again.session not in (0, client.session)
                expect(again, udp.ARM, (1, 0, 3))
                print("New session; ARM without a setpoint refused", flush=True)
            finally:
                again_offset = path.stat().st_size
                close(again)
            released(path, again_offset)
        elif mode == "LIMITS":
            print("Send vx=2.0 (limit 1.0): wheels must stop", flush=True)
            client.send(udp.SETPOINT, cdr(2.0))
            notification(client, 4)
            stop_event(path, offset, "OutOfLimits")
            recover(client, path, 4)
    finally:
        cleanup_offset = path.stat().st_size
        close(client)
    if mode != "BYE":
        released(path, cleanup_offset)
    print(f"UDP {mode}: protocol checks passed; wheel observation required", flush=True)


def check(path):
    assert udp.TARGET, "set ROVI_TARGET explicitly"
    assert path.is_file(), "provide the live serial capture"
    for mode in MODES:
        scenario(mode, path)
    print("UDP stop checks: passed; released", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--carlog", type=Path, required=True)
    check(parser.parse_args().carlog)
