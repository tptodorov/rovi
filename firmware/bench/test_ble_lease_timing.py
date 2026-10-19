"""BLE probe safeguards with a fake client/clock; no board timing claim."""
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import AsyncMock, patch

import ble_lease_timing as probe


class LeaseTiming(unittest.IsolatedAsyncioTestCase):
    async def run_probe(self, upper_ms=1010, reason="LeaseExpired", nonzero=False,
                        notify=True, release=True, stop_fails=False, mtu=247,
                        fresh_stop=True):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "car.log"
            # This old event must not make a new trial pass.
            path.write_text("ev=stop reason=LeaseExpired at_ms=1\n")
            car = SimpleNamespace(
                mtu=mtu, capabilities=SimpleNamespace(lease_ms=1000),
                peer=SimpleNamespace(is_connected=True),
                statuses=[("Stopped", "LeaseExpired", None)],
                status=AsyncMock(return_value=("Claimed", None, None)),
                send=AsyncMock(), stop=AsyncMock(return_value=True),
                wait_status=AsyncMock(return_value=("Stopped", "LeaseExpired", None)),
            )

            async def arm():
                wheels = "83,0,0,0" if nonzero else "0,0,0,0"
                with path.open("a") as output:
                    output.write(f"ev=state state=Armed wheels_pm={wheels} at_ms=100\n")
                    if fresh_stop:
                        output.write(f"ev=stop reason={reason} at_ms=1100\n")
                if notify:
                    car.statuses.append(("Stopped", "LeaseExpired", None))
                return True

            async def close():
                car.peer.is_connected = False
                if release:
                    with path.open("a") as output:
                        output.write("ev=released at_ms=1200\n")

            car.arm, car.close = AsyncMock(side_effect=arm), AsyncMock(side_effect=close)
            if stop_fails:
                car.stop.side_effect = RuntimeError("STOP write failed")
            # Each of five trials reads the laptop clock before send and after
            # serial observation; no wall-clock timing or real radio is involved.
            ticks = [t for n in range(5) for t in (
                n * 2_000_000_000, n * 2_000_000_000 + int(upper_ms * 1_000_000)
            )]
            with patch.object(probe.ble_client, "connect", AsyncMock(return_value=car)), \
                 patch.object(probe.time, "monotonic_ns", side_effect=ticks), \
                 patch.object(probe.carlog, "wait", return_value=(release, [])):
                try:
                    await probe.check(path)
                finally:
                    car.close.assert_awaited_once()
                    car.stop.assert_awaited_once()
                    for call in car.send.await_args_list:
                        self.assertEqual(call.args, (0.0,))
                    self.car = car

    async def test_five_zero_trials_and_release(self):
        await self.run_probe()
        self.assertEqual(self.car.send.await_count, 5)
        self.assertEqual(self.car.arm.await_count, 5)
        self.assertEqual(self.car.wait_status.await_count, 5)

    async def test_late_capture_fails_and_disconnects(self):
        with self.assertRaisesRegex(AssertionError, "exceeds lease\\+50"):
            await self.run_probe(upper_ms=1051)

    async def test_wrong_stop_cannot_pass(self):
        with self.assertRaises(AssertionError):
            await self.run_probe(reason="OwnerLost")

    async def test_nonzero_wheels_cannot_pass(self):
        with self.assertRaisesRegex(AssertionError, "nonzero wheel"):
            await self.run_probe(nonzero=True)

    async def test_missing_notification_cannot_pass(self):
        with self.assertRaisesRegex(AssertionError, "fresh BLE"):
            await self.run_probe(notify=False)

    async def test_old_serial_stop_cannot_pass(self):
        with self.assertRaisesRegex(AssertionError, "missing fresh serial"):
            await self.run_probe(fresh_stop=False)

    async def test_missing_release_cannot_pass(self):
        with self.assertRaisesRegex(AssertionError, "release"):
            await self.run_probe(release=False)

    async def test_failed_cleanup_stop_still_disconnects(self):
        with self.assertRaisesRegex(RuntimeError, "STOP write failed"):
            await self.run_probe(stop_fails=True)

    async def test_small_mtu_never_sends_or_arms(self):
        with self.assertRaises(AssertionError):
            await self.run_probe(mtu=23)
        self.car.send.assert_not_awaited()
        self.car.arm.assert_not_awaited()


if __name__ == "__main__":
    unittest.main()
