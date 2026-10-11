#!/usr/bin/env python3
"""A bench BLE client for the M8 GATT service (bleak). The BLE connection is the claim, so
connecting makes this laptop the owner.

    python3 sim/ble_client.py info                       # MTU, capabilities, status, then disconnect
    python3 sim/ble_client.py drive --vx 0.2 --seconds 2 # one setpoint, ARM, 20 Hz stream, STOP
    ROVI_BLE_ADDR=AA:BB:..    connect to this address instead of scanning for the name "Rovi M8"

Untried on a board: only the fake-peer tests in test_ble_client.py have run it.
"""
import argparse, asyncio, logging, os, time, warnings

import wire

try:
    from bleak.exc import BleakError
except ImportError:  # the unit tests run without bleak
    class BleakError(Exception):
        pass

SERVICE = "d47a0010-45b2-4d19-8db0-6bd87e9c0001"
SETPOINT = "d47a0011-45b2-4d19-8db0-6bd87e9c0001"  # write without response: seq u32 + CDR, 72 B
CONTROL = "d47a0012-45b2-4d19-8db0-6bd87e9c0001"  # write: 1 = ARM, 2 = STOP; refused with an ATT error
CAPABILITIES = "d47a0013-45b2-4d19-8db0-6bd87e9c0001"  # read, 27 B
STATUS = "d47a0014-45b2-4d19-8db0-6bd87e9c0001"  # read + notify: state, stop reason, last error
# HOOK(echo): the latency Echo characteristic, if the firmware has it. Its protocol belongs to
# sim/latency.py; this client only reports whether it is there (Car.has_echo).
ECHO = "d47a0015-45b2-4d19-8db0-6bd87e9c0001"
NAME = "Rovi M8"
MIN_MTU = 75  # 72-byte setpoint + 3 bytes of ATT header
ARM, STOP = 1, 2
log = logging.getLogger("rovi.ble")


def _adapter(adapter):
    return {"adapter": adapter} if adapter else {}  # a BlueZ adapter such as "hci1"


async def find(addr=None, timeout=10.0, adapter=None):
    """The car's BLEDevice, by address or by advertised name; None if not seen within timeout.
    The car does not advertise while another controller owns it."""
    from bleak import BleakScanner
    if addr:
        return await BleakScanner.find_device_by_address(addr, timeout=timeout, **_adapter(adapter))
    return await BleakScanner.find_device_by_name(NAME, timeout=timeout, **_adapter(adapter))


class Car:
    """One connection to the car. `peer` is a bleak BleakClient, or anything with its surface."""

    def __init__(self, peer):
        self.peer, self.seq, self.statuses = peer, 0, []
        self.mtu = self.capabilities = None
        self.has_echo = False

    async def open(self):
        if not self.peer.is_connected:
            await self.peer.connect()
        self.mtu = await self._mtu()
        if self.mtu < MIN_MTU:
            log.warning("ATT MTU %d < %d: the 72-byte setpoint does not fit (PLAN Q18)", self.mtu, MIN_MTU)
        self.capabilities = wire.Capabilities.decode(await self.peer.read_gatt_char(CAPABILITIES))
        self.has_echo = self.peer.services.get_characteristic(ECHO) is not None
        await self.peer.start_notify(STATUS, self._on_status)
        self.statuses.append(await self.status())
        return self

    async def _mtu(self):
        """The negotiated ATT MTU. BlueZ reports 23 until the MTU is acquired, and its write size
        can lag for a moment after connecting, so take the larger of the two views."""
        acquire = getattr(getattr(self.peer, "_backend", None), "_acquire_mtu", None)
        if acquire:
            try:
                await acquire()
            except Exception as e:  # not fatal: the write size below is the fallback
                log.debug("acquire MTU: %s", e)
        char = self.peer.services.get_characteristic(SETPOINT)
        if char is None:
            raise BleakError(f"no Setpoint characteristic {SETPOINT}: not an M8 firmware?")
        with warnings.catch_warnings():  # BlueZ warns on every mtu_size read before an acquire
            warnings.simplefilter("ignore")
            end = time.monotonic() + 2
            while self.peer.mtu_size <= 23 and char.max_write_without_response_size == 20 and time.monotonic() < end:
                await asyncio.sleep(0.2)
            return max(self.peer.mtu_size, char.max_write_without_response_size + 3)

    def _on_status(self, _char, data):
        self.statuses.append(wire.Status.decode(data))

    async def close(self):
        if self.peer.is_connected:
            try:
                await self.peer.stop_notify(STATUS)
            except BleakError:
                pass
            await self.peer.disconnect()

    async def status(self):
        return wire.Status.decode(await self.peer.read_gatt_char(STATUS))

    async def wait_status(self, pred, timeout):
        """The latest status, or a later notified one, that matches pred; None after timeout."""
        since, end = max(len(self.statuses) - 1, 0), time.monotonic() + timeout
        while time.monotonic() < end:
            hit = next((s for s in self.statuses[since:] if pred(s)), None)
            if hit:
                return hit
            await asyncio.sleep(0.01)
        return None

    async def send(self, vx=0.0, vy=0.0, wz=0.0):
        """One setpoint; returns its seq."""
        self.seq += 1
        await self.peer.write_gatt_char(SETPOINT, wire.ble_setpoint(self.seq, vx, vy, wz), response=False)
        return self.seq

    async def control(self, cmd):
        """Writes ARM or STOP; False if the car refused it (an ATT error)."""
        try:
            await self.peer.write_gatt_char(CONTROL, bytes([cmd]), response=True)
            return True
        except BleakError as e:
            log.info("control %d refused: %s", cmd, e)
            return False

    async def arm(self):
        return await self.control(ARM)

    async def stop(self):
        return await self.control(STOP)

    async def stream(self, vx=0.0, vy=0.0, wz=0.0, seconds=1.0, hz=20):
        """Setpoints at hz for seconds, on a fixed schedule; returns how many were sent."""
        n, period, t = 0, 1 / hz, time.monotonic()
        end = t + seconds
        while t < end:
            await self.send(vx, vy, wz)
            n += 1
            t += period
            await asyncio.sleep(max(0.0, t - time.monotonic()))
        return n


async def connect(addr=None, timeout=10.0, adapter=None):
    """Finds the car, connects (the claim) and opens it."""
    from bleak import BleakClient
    dev = await find(addr, timeout, adapter)
    if dev is None:
        raise BleakError(f"car not found ({addr or NAME!r}); is it advertising, or owned by someone else?")
    return await Car(BleakClient(dev, timeout=timeout, **_adapter(adapter))).open()


async def main(args):
    car = await connect(args.addr)
    try:
        print(f"connected: mtu={car.mtu} echo={car.has_echo}\n{car.capabilities}\nstatus: {car.statuses[-1]}")
        if args.cmd == "drive":
            await car.send(args.vx, args.vy, args.wz)
            if not await car.arm():
                print("arm refused")
                return 1
            n = await car.stream(args.vx, args.vy, args.wz, args.seconds)
            await car.stop()
            await asyncio.sleep(0.3)  # the car notifies status every 100 ms at most
            print(f"sent {n} setpoints; statuses: {car.statuses}")
    finally:
        await car.close()
    return 0


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("cmd", choices=("info", "drive"))
    p.add_argument("--addr", default=os.environ.get("ROVI_BLE_ADDR"))
    for k, d in (("--vx", 0.2), ("--vy", 0.0), ("--wz", 0.0), ("--seconds", 2.0)):
        p.add_argument(k, type=float, default=d)
    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")
    raise SystemExit(asyncio.run(main(p.parse_args())))
