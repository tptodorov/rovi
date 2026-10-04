#!/usr/bin/env python3
"""BLE bench driver for Rovi M3. Steps: hex byte(s) to write, 'sNNN' sleep ms, 'disc' disconnect."""
import asyncio, json, sys, time
from bleak import BleakClient, BleakScanner

CHAR = "d47a0002-45b2-4d19-8db0-6bd87e9c0001"


def log(**kw):
    print(json.dumps({"t_ns": time.monotonic_ns(), **kw}), flush=True)


async def main(steps):
    dev = await BleakScanner.find_device_by_name("Rovi M3", timeout=15)
    if dev is None:
        log(event="not_found"); return
    t0 = time.monotonic_ns()
    async with BleakClient(dev, timeout=20) as c:
        log(event="connected", connect_ms=(time.monotonic_ns() - t0) / 1e6, address=dev.address)
        for step in steps:
            if step == "disc":
                await c.disconnect(); log(event="disconnected_by_client"); return
            if step.startswith("s"):
                await asyncio.sleep(int(step[1:]) / 1000); continue
            if not c.is_connected:
                log(event="peer_disconnected_before", step=step); return
            data = bytes.fromhex(step)
            t = time.monotonic_ns()
            try:
                await c.write_gatt_char(CHAR, data, response=True)
                log(write=step, ok=True, rtt_ms=(time.monotonic_ns() - t) / 1e6)
            except Exception as e:
                log(write=step, ok=False, rtt_ms=(time.monotonic_ns() - t) / 1e6, error=str(e)[:120])
        log(event="steps_done", connected=c.is_connected)


asyncio.run(main(sys.argv[1:]))
