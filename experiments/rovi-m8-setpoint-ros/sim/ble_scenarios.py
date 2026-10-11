#!/usr/bin/env python3
"""The M8 BLE scenarios against a real board (bleak), mirroring sim/udp_scenarios.py. There is no
BLE sim car, so without a board every scenario is skipped.

    ROVI_BLE_ADDR=AA:BB:.. python3 sim/ble_scenarios.py     # the board, flashed and advertising "Rovi M8"
    ROVI_CARLOG=<file>        the board's serial capture, for the `ev=` checks (without it: client-side checks only)
    ROVI_BLE_ADAPTER2=hci1    a second BlueZ adapter, so a second central can try to connect. The bench has
                              one adapter, so that scenario stays skipped

The windows (lease, claim, reclaim) come from the car's Capabilities. Untried on a board.
"""
import asyncio, os, sys, unittest

import ble_client, carlog
from wire import Status
from carlog import ev

ADDR = os.environ.get("ROVI_BLE_ADDR")
CARLOG = os.environ.get("ROVI_CARLOG")
ADAPTER2 = os.environ.get("ROVI_BLE_ADAPTER2")
try:
    import bleak  # noqa: F401
    NO_BLEAK = None
except ImportError:
    NO_BLEAK = "bleak is missing: run inside the repo nix shell (shell.nix)"
SKIP = "no board: set ROVI_BLE_ADDR to the car's address" if not ADDR else NO_BLEAK


def armed(s):
    return s.state == "Armed"


def signs(wheels):
    return tuple((w > 0) - (w < 0) for w in wheels)


@unittest.skipIf(SKIP, SKIP)
class Scenarios(unittest.IsolatedAsyncioTestCase):
    lost = False  # the car was not found once: fail the rest at once instead of scanning again

    async def asyncSetUp(self):
        self.cars = []
        # A disconnect releases the car at once, but a test that failed mid-way may leave it owned
        # until its windows run out; wait for the car to advertise again.
        if Scenarios.lost or await ble_client.find(ADDR, timeout=30) is None:
            Scenarios.lost = True
            self.fail(f"the car {ADDR} is not advertising (owned by someone else, or off)")
        self.off = os.path.getsize(CARLOG) if CARLOG else 0

    async def asyncTearDown(self):
        for c in self.cars:
            await c.close()

    async def connect(self, adapter=None):
        car = await ble_client.connect(ADDR, timeout=10, adapter=adapter)
        self.cars.append(car)
        return car

    async def saw(self, pred):
        """The car's own log shows an event matching pred (within 1 s). Skipped without a car log."""
        if not CARLOG:
            return
        found, seen = await asyncio.to_thread(carlog.wait, CARLOG, self.off, pred)
        if not found:
            self.fail(f"car log lacks the event; last seen: {seen[-4:]}")

    async def status(self, car, pred, timeout=1.0, msg=None):
        s = await car.wait_status(pred, timeout)
        self.assertIsNotNone(s, msg or f"status never matched; seen: {car.statuses[-4:]}")
        return s

    async def armed_car(self):
        car = await self.connect()
        await car.send(0.2)
        self.assertTrue(await car.arm())
        await self.status(car, armed)
        return car

    def secs(self, car, name):
        return getattr(car.capabilities, name) / 1000

    async def test_claim_by_connect_arm_needs_a_fresh_setpoint_drive_stop(self):
        car = await self.connect()
        self.assertGreaterEqual(car.mtu, ble_client.MIN_MTU, "a 72-byte setpoint needs ATT MTU >= 75 (Q18)")
        self.assertEqual(car.statuses[0], Status("Claimed", None, None), "connecting is the claim; Status is set at connect")
        self.assertFalse(await car.arm(), "arm needs a fresh setpoint: the write is rejected")
        await self.status(car, lambda s: s.error == "NotReady", msg="a refused ARM shows NotReady in Status")
        await car.send(0.2)
        self.assertTrue(await car.arm())
        await self.status(car, lambda s: s == ("Armed", None, None), msg="a successful ARM clears the error")
        await car.stream(0.2, seconds=self.secs(car, "lease_ms") * 1.5)  # longer than one lease
        self.assertEqual(car.statuses[-1].state, "Armed", "the stream renews the lease")
        self.assertTrue(await car.stop())
        self.assertEqual(await self.status(car, lambda s: s.state == "Stopped"), ("Stopped", "Stop", None))
        self.assertFalse(await car.arm(), "arming after a stop needs a setpoint sent after it")
        await self.status(car, lambda s: s.error == "NotReady")
        await car.send(0.2)
        self.assertTrue(await car.arm())
        await self.status(car, lambda s: s == ("Armed", None, None))
        await self.saw(ev("claimed", source="Ble"))
        await self.saw(lambda e: e.get("ev") == "ble" and e.get("link") == "up" and e.get("mtu", 0) >= ble_client.MIN_MTU)
        await self.saw(ev("wifi", ap="off"))
        await self.saw(ev("state", state="Armed"))
        await self.saw(ev("stop", reason="Stop"))

    async def test_lease_expiry_then_rearm_inside_the_reclaim_window(self):
        car = await self.armed_car()
        stopped = await self.status(car, lambda s: s.state == "Stopped", timeout=self.secs(car, "lease_ms") + 0.5)
        self.assertEqual(stopped.reason, "LeaseExpired")
        await self.saw(ev("stop", reason="LeaseExpired"))
        await car.send(0.2)
        self.assertTrue(await car.arm(), "the same owner re-arms inside the reclaim window")
        await self.status(car, armed)
        self.assertTrue(car.peer.is_connected)

    async def test_unknown_control_is_rejected_with_invalid(self):
        car = await self.connect()
        self.assertFalse(await car.control(9))
        await self.status(car, lambda s: s == ("Claimed", None, "Invalid"))
        await car.send(0.2)
        self.assertTrue(await car.arm())
        await self.status(car, lambda s: s == ("Armed", None, None))

    async def test_stop_keeps_the_owner_while_it_keeps_streaming(self):
        car = await self.armed_car()
        self.assertTrue(await car.stop())
        await car.stream(0, seconds=self.secs(car, "lease_ms") + self.secs(car, "reclaim_ms") + 0.5)
        self.assertTrue(car.peer.is_connected, "no reclaim window runs while the owner streams")
        self.assertEqual(car.statuses[-1], ("Stopped", "Stop", None))
        await car.send(0.2)
        self.assertTrue(await car.arm())
        await self.status(car, armed)

    async def test_stop_then_silence_releases_after_the_lease_and_the_reclaim_window(self):
        car = await self.armed_car()
        self.assertTrue(await car.stop())
        await asyncio.sleep(self.secs(car, "lease_ms") + self.secs(car, "reclaim_ms") + 0.7)
        self.assertFalse(car.peer.is_connected, "the reclaim window starts when the lease lapses")
        await self.saw(ev("released"))

    async def test_reclaim_window_end_releases_the_owner(self):
        car = await self.armed_car()
        await asyncio.sleep(self.secs(car, "lease_ms") + self.secs(car, "reclaim_ms") + 0.5)
        self.assertFalse(car.peer.is_connected, "the car drops the link when it releases the owner")
        await self.saw(ev("released"))

    async def test_claim_window_expires_if_the_owner_stays_silent(self):
        car = await self.connect()
        await asyncio.sleep(self.secs(car, "claim_ms") + 0.5)
        self.assertFalse(car.peer.is_connected)
        await self.saw(ev("released"))
        self.assertIsNotNone(await ble_client.find(ADDR, timeout=10), "the car advertises again")

    async def test_disconnect_releases_and_the_car_advertises_again(self):
        car = await self.connect()
        self.assertFalse(await car.arm())
        await self.status(car, lambda s: s.error == "NotReady")
        await car.send(0.2)
        self.assertTrue(await car.arm())
        await car.close()
        await self.saw(ev("released"))
        await self.saw(lambda e: e.get("ev") == "ble" and e.get("link") == "down")
        await self.saw(ev("wifi", ap="on"))
        again = await self.connect()
        self.assertEqual(again.statuses[0], Status("Claimed", None, None), "a new owner starts disarmed, without an error")

    async def test_second_central_cannot_connect_while_owned(self):
        if not ADAPTER2:
            self.skipTest("set ROVI_BLE_ADAPTER2 to a second BlueZ adapter (one adapter is one central)")
        car = await self.connect()
        with self.assertRaises((ble_client.BleakError, asyncio.TimeoutError)):
            await self.connect(adapter=ADAPTER2)
        self.assertTrue(car.peer.is_connected, "the owner keeps the car")
        await car.send(0.2)
        self.assertTrue(await car.arm())

    async def test_setpoints_reach_the_wheels_with_mecanum_signs(self):
        car = await self.armed_car()
        for (vx, vy, wz), want in [((0.5, 0, 0), (1, 1, 1, 1)), ((0, 0.5, 0), (-1, 1, -1, 1)), ((0, 0, 1), (-1, 1, 1, -1))]:
            await car.stream(vx, vy, wz, seconds=0.5)
            await self.saw(lambda e: e.get("ev") == "state" and e["state"] == "Armed" and signs(e["wheels_pm"]) == want)
        self.assertTrue(await car.stop())

    # HOOK(echo): BLE round-trip latency (Q19) belongs to sim/latency.py, which owns the Echo
    # characteristic's protocol; here a board run only records whether the firmware has it.
    async def test_echo_characteristic_presence_is_reported(self):
        car = await self.connect()
        print(f"\nmtu={car.mtu} echo={car.has_echo} {car.capabilities}", file=sys.stderr)


if __name__ == "__main__":
    unittest.main(argv=[sys.argv[0], "-v"])
