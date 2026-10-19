#!/usr/bin/env python3
"""BLE lease timing at zero wheels; board runs require a separately announced go.

Start serial capture, then:
    nix-shell --run 'python3 firmware/bench/ble_lease_timing.py --carlog <live-car.log>'
"""
import argparse
import asyncio
from pathlib import Path
import time

from ble_stop_checks import ble_client, carlog


async def trial(car, path):
    lease_ms = car.capabilities.lease_ms
    offset, notify_offset = path.stat().st_size, len(car.statuses)
    # The sole setpoint must be accepted for ARM to succeed. Both clock reads
    # are on the laptop; BLE/capture/polling delays enlarge the measured bound.
    started = time.monotonic_ns()
    await car.send(0.0)
    assert await car.arm(), "zero setpoint did not arm"
    deadline = started + (lease_ms + 1050) * 1_000_000
    while True:
        events = carlog.events(path, offset)
        observed = time.monotonic_ns()
        assert not any(e.get("ev") in ("boot", "error", "dropped", "released")
                       for e in events), events
        assert all(e.get("wheels_pm", [0, 0, 0, 0]) == [0, 0, 0, 0]
                   for e in events), "nonzero wheel command"
        stops = [e for e in events if e.get("ev") == "stop"]
        if stops:
            assert stops[0].get("reason") == "LeaseExpired", stops
            assert any(carlog.ev("state", state="Armed")(e) for e in events), events
            upper_ms = (observed - started) / 1_000_000
            assert upper_ms <= lease_ms + 50, (
                f"observed upper bound {upper_ms:.3f} ms exceeds lease+50 "
                f"({lease_ms + 50} ms); includes BLE/serial/host delay"
            )
            status = await car.wait_status(lambda s: s.reason == "LeaseExpired", timeout=0.5)
            expected = ("Stopped", "LeaseExpired", None)
            assert status == expected and expected in car.statuses[notify_offset:], \
                "missing fresh BLE lease-expired Status notification"
            return upper_ms
        assert observed < deadline, "missing fresh serial lease stop"
        await asyncio.sleep(0.002)


async def check(path, trials=5):
    assert path.is_file(), "provide the live serial capture"
    assert trials > 0, "at least one trial is required"
    car = await ble_client.connect()
    try:
        assert car.mtu >= ble_client.MIN_MTU, car.mtu
        assert car.capabilities.lease_ms == 1000, car.capabilities
        assert await car.status() == ("Claimed", None, None)
        results = []
        for n in range(trials):
            upper_ms = await trial(car, path)
            results.append(upper_ms)
            print(f"Trial {n + 1}/{trials}: upper_bound_ms={upper_ms:.3f} "
                  f"lease_ms=1000 limit_ms=1050; zero wheels", flush=True)
    finally:
        cleanup_offset = path.stat().st_size
        try:
            if car.peer.is_connected:
                async with asyncio.timeout(3):
                    assert await car.stop(), "cleanup STOP refused"
        finally:
            try:
                async with asyncio.timeout(5):
                    await car.close()
            finally:
                found, _ = await asyncio.to_thread(
                    carlog.wait, path, cleanup_offset, carlog.ev("released"), timeout=3
                )
                assert found, "cleanup release was not observed in serial"
    print(f"BLE lease timing: passed; max_upper_bound_ms={max(results):.3f}; released",
          flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--carlog", type=Path, required=True)
    parser.add_argument("--trials", type=int, default=5)
    args = parser.parse_args()
    asyncio.run(check(args.carlog, args.trials))
