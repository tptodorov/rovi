"""Only probe cleanup/profile guards; these tests do not establish board behavior."""
import unittest
from pathlib import Path
import tempfile
from types import SimpleNamespace
from unittest.mock import AsyncMock, patch

import ble_stop_checks as probe


class Cleanup(unittest.IsolatedAsyncioTestCase):
    def car(self):
        return SimpleNamespace(
            mtu=247,
            capabilities=SimpleNamespace(max_vx=1.0, lease_ms=1000),
            peer=SimpleNamespace(is_connected=True),
            status=AsyncMock(side_effect=[("Claimed", None, None), ("Armed", None, None)]),
            send=AsyncMock(),
            arm=AsyncMock(return_value=True),
            stream=AsyncMock(side_effect=RuntimeError("stream failed")),
            stop=AsyncMock(return_value=True),
            close=AsyncMock(),
        )

    async def run_scenario(self, car):
        with patch.object(probe.ble_client, "connect", AsyncMock(return_value=car)), patch.object(
            probe.asyncio, "sleep", AsyncMock()
        ):
            await probe.scenario("STOP")

    async def test_failure_while_driving_still_stops_and_disconnects(self):
        car = self.car()
        with self.assertRaisesRegex(RuntimeError, "stream failed"):
            await self.run_scenario(car)
        car.arm.assert_awaited_once()
        car.stop.assert_awaited_once()
        car.close.assert_awaited_once()

    async def test_failed_cleanup_stop_still_disconnects(self):
        car = self.car()
        car.stop.side_effect = RuntimeError("stop write failed")
        with self.assertRaisesRegex(RuntimeError, "stop write failed"):
            await self.run_scenario(car)
        car.close.assert_awaited_once()

    async def test_unannounced_profile_never_sends_a_setpoint_or_arms(self):
        car = self.car()
        car.capabilities.max_vx = 0.5
        with self.assertRaises(AssertionError):
            await self.run_scenario(car)
        car.send.assert_not_awaited()
        car.arm.assert_not_awaited()
        car.close.assert_awaited_once()

    async def disconnect_case(self, reason):
        car, again = self.car(), self.car()
        car.stream.side_effect = None
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "car.log"
            log.write_text("ev=stop reason=OwnerLost at_ms=1\n")

            async def stop_event():
                with log.open("a") as output:
                    output.write(f"ev=stop reason={reason} at_ms=4100\n")
                car.peer.is_connected = False

            car.close.side_effect = stop_event
            car.peer.disconnect = AsyncMock(side_effect=stop_event)
            with patch.object(
                probe.ble_client, "connect", AsyncMock(side_effect=[car, again])
            ), patch.object(probe.asyncio, "sleep", AsyncMock()):
                await probe.scenario("DISCONNECT", log)
            car.peer.disconnect.assert_awaited_once()
            again.arm.assert_not_awaited()
            again.close.assert_awaited_once()

    async def test_lease_stop_during_disconnect_cannot_pass_as_owner_loss(self):
        with self.assertRaisesRegex(AssertionError, "OwnerLost"):
            await self.disconnect_case("LeaseExpired")

    async def test_owner_loss_allows_disarmed_reconnect(self):
        await self.disconnect_case("OwnerLost")


if __name__ == "__main__":
    unittest.main()
