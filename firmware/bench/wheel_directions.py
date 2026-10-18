#!/usr/bin/env python3
"""Physical wheel/direction check over UDP; run only after an announced test and Todor's go.

Stand, wheels clear, motor bank awake; join Rovi's AP and capture serial first:
    ROVI_TARGET=192.168.4.1:7777 python3 firmware/bench/wheel_directions.py --carlog <live-car.log>

One armed session, six gentle moves of 3 s, each followed by 3 s of zero setpoints.
Forward, strafe and rotate give every wheel a distinct sign pattern, so a swapped
or inverted wheel is visible. Serial must show the expected wheel signs; the
physical direction of each wheel is Todor's observation.
"""
import argparse
from pathlib import Path
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "apps/simcar/scenarios"))
import carlog
import udp_scenarios as udp
from wire import cdr

LEAD_SECONDS, DRIVE_SECONDS, PAUSE_SECONDS = 5, 3, 3
# (name, vx, vy, wz, expected signs FL, FR, RR, RL); about 83-85 permille per wheel.
MOVES = (
    ("forward", 0.1, 0, 0, (1, 1, 1, 1)),
    ("backward", -0.1, 0, 0, (-1, -1, -1, -1)),
    ("strafe left", 0, 0.1, 0, (-1, 1, -1, 1)),
    ("strafe right", 0, -0.1, 0, (1, -1, 1, -1)),
    ("rotate left (CCW from above)", 0, 0, 0.8, (-1, 1, 1, -1)),
    ("rotate right (CW from above)", 0, 0, -0.8, (1, -1, -1, 1)),
)


def stream(client, seconds, vx=0.0, vy=0.0, wz=0.0):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        client.send(udp.SETPOINT, cdr(vx, vy, wz))
        time.sleep(0.05)


def sign(v):
    return (v > 0) - (v < 0)


def check(path):
    assert udp.TARGET, "set ROVI_TARGET explicitly"
    assert path.is_file(), "provide the live serial capture"
    print(f"Wheel directions: first move in {LEAD_SECONDS} s", flush=True)
    time.sleep(LEAD_SECONDS)
    client = udp.Client()
    try:
        hello = client.hello()
        assert hello and hello[0] == udp.ACK and hello[1] != 0, hello
        client.send(udp.SETPOINT, cdr())
        assert client.status(udp.ARM) == (2, 0, 0), "ARM refused"
        for number, (name, vx, vy, wz, signs) in enumerate(MOVES, 1):
            offset = path.stat().st_size
            print(f"{number}/6 {name}: 3 s", flush=True)
            stream(client, DRIVE_SECONDS, vx, vy, wz)
            found, seen = carlog.wait(path, offset, carlog.ev("state", state="Armed"))
            moving = [e["wheels_pm"] for e in seen
                      if carlog.ev("state", state="Armed")(e) and any(e["wheels_pm"])]
            assert found and moving, (name, seen[-4:])
            wheels = moving[-1]
            assert tuple(map(sign, wheels)) == signs, (name, wheels, signs)
            print(f"  serial wheels_pm FL,FR,RR,RL = {wheels}", flush=True)
            offset = path.stat().st_size
            stream(client, PAUSE_SECONDS)
            found, _ = carlog.wait(path, offset, carlog.ev("state", state="Armed",
                                                            wheels_pm=[0, 0, 0, 0]))
            assert found, (name, "did not return to zero")
            print("  zero: wheels still", flush=True)
        assert client.status(udp.STOP) == (3, 1, 0), "STOP refused"
    finally:
        offset = path.stat().st_size
        try:
            client.send(udp.STOP)
            client.send(udp.BYE)
        finally:
            client.s.close()
    found, _ = carlog.wait(path, offset, carlog.ev("released"))
    assert found, "release was not observed in serial"
    print("Wheel directions: serial signs passed; released; wheel observation required", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--carlog", type=Path, required=True)
    check(parser.parse_args().carlog)
