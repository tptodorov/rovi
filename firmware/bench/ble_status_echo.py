#!/usr/bin/env python3
"""Board-only BLE Status/Echo acceptance probe; never sends a setpoint or a successful ARM.

Run only after an announced/authorized flash/bench session:
    nix-shell --run 'python3 firmware/bench/ble_status_echo.py'
Keep the motor bank disconnected for this first, non-driving probe.
"""
import asyncio

from bleak import BleakClient, BleakScanner
from bleak.exc import BleakError

SERVICE = "d47a0010-45b2-4d19-8db0-6bd87e9c0001"
CONTROL = "d47a0012-45b2-4d19-8db0-6bd87e9c0001"
STATUS = "d47a0014-45b2-4d19-8db0-6bd87e9c0001"
ECHO = "d47a0015-45b2-4d19-8db0-6bd87e9c0001"


async def check():
    # Firmware advertises its name in scan response, without the service UUID.
    device = await BleakScanner.find_device_by_name("Rovi M8", timeout=10)
    assert device is not None, "Rovi not found"
    async with BleakClient(device) as client:
        assert client.services.get_service(SERVICE) is not None
        # First read precedes even the subscription writes.
        assert bytes(await client.read_gatt_char(STATUS)) == bytes([1, 0, 0])
        print("immediate claimed/clear read: ok", flush=True)
        # BlueZ needs an acquire to expose the MTU (same path as the existing bench client).
        acquire = getattr(client._backend, "_acquire_mtu", None)
        if acquire:
            await acquire()
        assert client.mtu_size >= 35, "32-byte Echo needs ATT MTU >= 35"
        statuses, echoes = asyncio.Queue(), asyncio.Queue()
        await client.start_notify(STATUS, lambda _, data: statuses.put_nowait(bytes(data)))
        # BlueZ AcquireNotify treats an empty packet as EOF; D-Bus preserves it.
        await client.start_notify(
            ECHO,
            lambda _, data: echoes.put_nowait(bytes(data)),
            bluez={"use_start_notify": True},
        )

        async def expect_status(expected):
            assert bytes(await client.read_gatt_char(STATUS)) == expected
            async with asyncio.timeout(1):
                while await statuses.get() != expected:
                    pass
            print(f"read + notification {list(expected)}: ok", flush=True)

        async def refused(command):
            try:
                await client.write_gatt_char(CONTROL, bytes([command]), response=True)
            except BleakError:
                return
            raise AssertionError(f"control {command} should be refused")

        async def echo(payload, expected_status):
            await client.write_gatt_char(ECHO, payload, response=True)
            reply = await asyncio.wait_for(echoes.get(), 1)
            assert reply == payload, (payload, reply)
            assert bytes(await client.read_gatt_char(STATUS)) == expected_status
            print(f"echo {len(payload)} bytes, error retained: ok", flush=True)

        await refused(0)  # malformed owner command; no setpoint or lease
        await expect_status(bytes([1, 0, 4]))
        for payload in (b"", b"rovi", bytes(range(32))):
            await echo(payload, bytes([1, 0, 4]))
        await client.write_gatt_char(CONTROL, b"\x02", response=True)
        await expect_status(bytes([3, 1, 0]))
        await refused(1)  # ARM without a setpoint must never enable
        await expect_status(bytes([3, 1, 3]))
        await echo(b"", bytes([3, 1, 3]))
        await client.write_gatt_char(CONTROL, b"\x02", response=True)
        await expect_status(bytes([3, 1, 0]))
    print("BLE Status/Echo probe: passed; disconnected", flush=True)


if __name__ == "__main__":
    asyncio.run(check())
