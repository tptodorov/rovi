#!/usr/bin/env python3
"""BLE/AP radio exclusion on the board; run only after an announced test and Todor's go.

Never sends ARM: the only setpoints are zeros that hold a BLE or UDP claim
past the 10 s claim window. Start a live
serial capture, leave the laptop Wi-Fi off the rover's AP, then:
    nix-shell --run 'python3 firmware/bench/radio_exclusion.py --carlog <live-car.log>'

1. BLE connect claims: AP goes off (serial, and joining `rovi-m8` fails).
   BLE disconnect releases: AP comes back (serial, join works, UDP HELLO ACK).
2. UDP HELLO claims: BLE advertising stops (no advert in a fresh scan).
   BYE releases: advertising returns.
The caller restores the laptop's prior Wi-Fi connection.
"""
import argparse
import asyncio
import os
from pathlib import Path
import subprocess
import sys

from bleak import BleakClient, BleakScanner

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "apps/simcar/scenarios"))
import carlog

NAME, PROFILE, TARGET = "Rovi M8", "rovi-m8", "192.168.4.1:7777"
SERVICE = "d47a0010-45b2-4d19-8db0-6bd87e9c0001"
SETPOINT = "d47a0011-45b2-4d19-8db0-6bd87e9c0001"
SCAN_SECONDS = 8


def serial(path, offset, name, timeout=5, **fields):
    found, seen = carlog.wait(path, offset, carlog.ev(name, **fields), timeout=timeout)
    assert found, (name, fields, [e.get("ev") for e in seen[-6:]])
    print(f"  serial ev={name} {fields}", flush=True)


def join(timeout):
    result = subprocess.run(["nmcli", "--wait", str(timeout), "connection", "up", PROFILE],
                            capture_output=True, text=True)
    return result.returncode == 0


async def advertising():
    return await BleakScanner.find_device_by_name(NAME, timeout=SCAN_SECONDS) is not None


async def check(path):
    os.environ["ROVI_TARGET"] = TARGET
    import udp_scenarios as udp  # reads ROVI_TARGET at import
    from wire import ble_setpoint, cdr

    print("1. BLE owner turns the AP off", flush=True)
    device = await BleakScanner.find_device_by_name(NAME, timeout=10)
    assert device is not None, "Rovi not advertising while open"
    offset = path.stat().st_size
    async with BleakClient(device) as client:
        assert client.services.get_service(SERVICE) is not None
        serial(path, offset, "claimed", source="Ble")
        serial(path, offset, "wifi", ap="off")
        stop = asyncio.Event()

        async def hold_ble():  # zero, disarmed setpoints keep the claim; never ARM
            seq = 0
            while not stop.is_set():
                seq += 1
                await client.write_gatt_char(SETPOINT, ble_setpoint(seq), response=False)
                await asyncio.sleep(0.2)

        holder = asyncio.create_task(hold_ble())
        try:
            joined = await asyncio.to_thread(join, 15)
            assert not joined, "joined the rover AP while BLE owns the car"
            print("  rover AP not joinable (15 s attempt)", flush=True)
            assert not any(carlog.ev("released")(e) for e in carlog.events(path, offset)), \
                "BLE claim lapsed during the join attempt"
        finally:
            stop.set()
            await holder
        offset = path.stat().st_size
    serial(path, offset, "released")
    serial(path, offset, "wifi", ap="on")
    for _ in range(3):
        if await asyncio.to_thread(join, 15):
            break
    else:
        raise AssertionError("rover AP did not come back")
    client = udp.Client()
    try:
        hello = client.hello()
        assert hello and hello[0] == udp.ACK, hello
        print("  AP back: joined, UDP HELLO ACK", flush=True)

        print("2. UDP owner stops BLE advertising", flush=True)
        stop = asyncio.Event()

        async def hold():  # zero, disarmed setpoints keep the claim; never ARM
            while not stop.is_set():
                client.send(udp.SETPOINT, cdr())
                await asyncio.sleep(0.2)

        holder = asyncio.create_task(hold())
        try:
            assert not await advertising(), "Rovi advertises while UDP owns the car"
            print(f"  no advert in {SCAN_SECONDS} s scan", flush=True)
        finally:
            stop.set()
            await holder
        offset = path.stat().st_size
    finally:
        client.send(udp.BYE)
        client.s.close()
    serial(path, offset, "released")
    assert await advertising(), "advertising did not return after release"
    print("  released: advertising again", flush=True)
    print("Radio exclusion: passed; car open", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--carlog", type=Path, required=True)
    path = parser.parse_args().carlog
    assert path.is_file(), "provide the live serial capture"
    asyncio.run(check(path))
