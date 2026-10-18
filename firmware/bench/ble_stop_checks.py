#!/usr/bin/env python3
"""Board acceptance subset: BLE stops and explicit recovery, with visible motion.

Run only after an announced test and Todor's go, wheels clear, motor bank awake:
    nix-shell --run 'python3 firmware/bench/ble_stop_checks.py'

Uses the frozen M8 transport client; assertions below check the product contract.
Status/log evidence needs separate physical confirmation that all four wheels stop.
"""
import asyncio
from pathlib import Path
import sys
import time

sys.path.insert(
    0, str(Path(__file__).resolve().parents[2] / "experiments/rovi-m8-setpoint-ros/sim")
)
import ble_client


async def expect(car, state, reason=None, error=None):
    status = await car.status()
    assert status == (state, reason, error), status
    print(f"Status: {status}", flush=True)


async def recover(car, reason):
    assert not await car.arm(), "recovery must refuse ARM before a fresh setpoint"
    await expect(car, "Stopped", reason, "NotReady")
    print("Fresh gentle stream for 1.5 s: wheels must stay stopped", flush=True)
    await car.stream(0.1, seconds=1.5)
    await expect(car, "Stopped", reason, "NotReady")
    print("Explicit ARM: gentle forward for 3 s, then STOP", flush=True)
    assert await car.arm()
    await expect(car, "Armed")
    await car.stream(0.1, seconds=3)
    assert await car.stop()
    await expect(car, "Stopped", "Stop")


async def scenario(mode):
    print(f"\n=== {mode}: starts in 5 s ===", flush=True)
    await asyncio.sleep(5)
    car = await ble_client.connect()
    try:
        assert car.mtu >= ble_client.MIN_MTU
        # Keep the announced speeds/timings tied to the profile being accepted.
        assert car.capabilities.max_vx == 1.0 and car.capabilities.lease_ms == 1000
        await expect(car, "Claimed")
        await car.send(0.1)
        assert await car.arm()
        await expect(car, "Armed")
        print("Gentle forward vx=0.1 for 3 s", flush=True)
        await car.stream(0.1, seconds=3)

        if mode == "STOP":
            assert await car.stop()
            await expect(car, "Stopped", "Stop")
            await recover(car, "Stop")
        elif mode == "LEASE":
            print("Laptop silent: wheels must stop after about 1 s", flush=True)
            start = time.monotonic()
            stopped = await car.wait_status(
                lambda status: status.state == "Stopped", timeout=1.5
            )
            assert stopped == ("Stopped", "LeaseExpired", None), stopped
            print(f"Lease notification after stream: {(time.monotonic()-start)*1000:.0f} ms", flush=True)
            # Delivery timing here does not establish the lease+50 ms detection bound.
            await expect(car, "Stopped", "LeaseExpired")
            await recover(car, "LeaseExpired")
        elif mode == "DISCONNECT":
            print("Disconnect while driving: wheels must stop", flush=True)
            await car.close()
            await asyncio.sleep(1)
            car = await ble_client.connect()
            await expect(car, "Claimed")
            print("Reconnected disarmed, without a setpoint", flush=True)
        elif mode == "LIMITS":
            print("Send vx=2.0 (limit is 1.0): wheels must stop", flush=True)
            await car.send(2.0)
            stopped = await car.wait_status(
                lambda status: status.reason == "OutOfLimits", timeout=1
            )
            assert stopped == ("Stopped", "OutOfLimits", None), stopped
            await expect(car, "Stopped", "OutOfLimits")
            await recover(car, "OutOfLimits")
        else:
            raise AssertionError(mode)
    finally:
        try:
            if car.peer.is_connected:
                async with asyncio.timeout(3):
                    if not await car.stop():
                        raise RuntimeError("cleanup STOP refused")
        finally:
            await car.close()
    print(f"{mode}: software checks passed; wheel observation still required", flush=True)


async def check():
    for mode in ("STOP", "LEASE", "DISCONNECT", "LIMITS"):
        await scenario(mode)
    print("BLE stop checks: passed; disconnected", flush=True)


if __name__ == "__main__":
    asyncio.run(check())
