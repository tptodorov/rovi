"""The BLE client against an in-memory GATT peer (no radio, no bleak needed)."""
import asyncio, os, pathlib, struct, subprocess, sys, time, unittest

import ble_client, wire

CAPS = bytes([1]) + struct.pack("<5f", 0.05, 0.2, 1.0, 1.0, 3.0) + struct.pack("<3H", 1000, 10000, 5000)


class Char:
    def __init__(self, uuid, properties, mwwr=20):
        self.uuid, self.properties, self.max_write_without_response_size = uuid, properties, mwwr


class Services:
    def __init__(self, chars):
        self.chars = {c.uuid: c for c in chars}

    def get_characteristic(self, uuid):
        return self.chars.get(uuid)


class FakePeer:
    """The surface of bleak's BleakClient that the client uses, acting like the M8 firmware's GATT
    service. Status is set at connect. Control is a write with response, rejected (as an ATT error)
    when ARM comes without a setpoint in the last lease or since a STOP, or for an unknown command;
    the refusal shows in the Status error byte (NotReady, Invalid) until a control write succeeds.
    Status changes are notified."""

    lease = 1.0

    def __init__(self, mtu=247, mwwr=None, echo=False):
        self.mtu_size, self.is_connected = mtu, False
        chars = [Char(ble_client.SETPOINT, ["write-without-response"], mwwr if mwwr is not None else mtu - 3),
                 Char(ble_client.CONTROL, ["write"]), Char(ble_client.CAPABILITIES, ["read"]),
                 Char(ble_client.STATUS, ["read", "notify"])]
        if echo:
            chars.append(Char(ble_client.ECHO, ["write", "notify"]))
        self.services = Services(chars)
        self.writes, self.reads, self.notify, self.state, self.last_setpoint = [], [], {}, [0, 0, 0], None
        self.fresh_needed = False

    async def connect(self):
        self.is_connected, self.state = True, [1, 0, 0]  # connecting is the claim

    async def disconnect(self):
        self.is_connected, self.state = False, [0, 0, 0]

    async def read_gatt_char(self, uuid):
        self.reads.append(uuid)
        return bytearray({ble_client.CAPABILITIES: CAPS, ble_client.STATUS: bytes(self.state)}[uuid])

    async def start_notify(self, uuid, cb):
        self.notify[uuid] = cb

    async def stop_notify(self, uuid):
        self.notify.pop(uuid)

    async def write_gatt_char(self, uuid, data, response=False):
        self.writes.append((uuid, bytes(data), response, time.monotonic()))
        if uuid == ble_client.SETPOINT:
            self.last_setpoint, self.fresh_needed = time.monotonic(), False
        elif uuid == ble_client.CONTROL:
            assert response, "Control is a write with response"
            fresh = self.last_setpoint and time.monotonic() - self.last_setpoint < self.lease and not self.fresh_needed
            if data == b"\x01" and fresh:
                self.set([2, 0, 0])
            elif data == b"\x02":
                self.set([3, 1, 0])
                self.fresh_needed = True
            else:
                self.set(self.state[:2] + [3 if data == b"\x01" else 4])  # NotReady, Invalid
                raise ble_client.BleakError("ATT error 0x13 (Value Not Allowed)")

    def set(self, state):
        self.state = state
        if ble_client.STATUS in self.notify:
            self.notify[ble_client.STATUS](None, bytearray(state))


def run(coro):
    return asyncio.run(coro)


class Client(unittest.TestCase):
    def open(self, peer):
        async def go():
            return await ble_client.Car(peer).open()
        return run(go())

    def test_open_reads_capabilities_reports_mtu_and_subscribes(self):
        peer = FakePeer()
        car = self.open(peer)
        self.assertEqual(car.mtu, 247)
        self.assertEqual(car.capabilities.lease_ms, 1000)
        self.assertEqual(car.statuses, [wire.Status("Claimed", None, None)], "status is read once on open")
        self.assertIn(ble_client.STATUS, peer.notify)
        self.assertFalse(car.has_echo)

    def test_status_is_read_first_and_is_right_without_a_notification(self):
        peer = FakePeer()
        peer.start_notify = lambda uuid, cb: asyncio.sleep(0)  # the board sends no notification yet
        car = self.open(peer)
        self.assertEqual(peer.reads[0], ble_client.STATUS, "Status is read right after connecting")
        self.assertEqual(car.statuses[0], wire.Status("Claimed", None, None))

    def test_refused_controls_show_in_the_error_byte_until_a_control_write_succeeds(self):
        peer = FakePeer()

        async def go():
            car = await ble_client.Car(peer).open()
            self.assertFalse(await car.arm())
            self.assertEqual(await car.status(), ("Claimed", None, "NotReady"))
            self.assertFalse(await car.control(9))
            self.assertEqual(await car.status(), ("Claimed", None, "Invalid"))
            await car.send(0.1)
            self.assertEqual((await car.status()).error, "Invalid", "a setpoint does not clear it")
            self.assertTrue(await car.arm())
            return await car.status()
        self.assertEqual(run(go()), ("Armed", None, None))

    def test_arm_after_stop_needs_a_setpoint_sent_after_it(self):
        peer = FakePeer()

        async def go():
            car = await ble_client.Car(peer).open()
            await car.send(0.1)
            self.assertTrue(await car.arm())
            self.assertTrue(await car.stop())
            self.assertFalse(await car.arm(), "the lease is still valid, but no setpoint since the stop")
            self.assertEqual(await car.status(), ("Stopped", "Stop", "NotReady"))
            await car.send(0.1)
            self.assertTrue(await car.arm())
        run(go())

    def test_warns_when_mtu_cannot_carry_a_setpoint(self):
        with self.assertLogs("rovi.ble", "WARNING") as logs:
            car = self.open(FakePeer(mtu=23))
        self.assertEqual(car.mtu, 23)
        self.assertIn("MTU 23 < 75", logs.output[0])

    def test_mtu_falls_back_to_the_write_size_when_bluez_reports_23(self):
        car = self.open(FakePeer(mtu=23, mwwr=244))
        self.assertEqual(car.mtu, 247)

    def test_echo_characteristic_is_detected(self):
        self.assertTrue(self.open(FakePeer(echo=True)).has_echo)

    def test_setpoints_are_seq_then_cdr_without_response_and_seq_increases(self):
        peer = FakePeer()

        async def go():
            car = await ble_client.Car(peer).open()
            await car.send(0.5)
            await car.send(0, 0.2, 1.0)
        run(go())
        (u1, d1, r1, _), (_, d2, _, _) = peer.writes
        self.assertEqual((u1, r1), (ble_client.SETPOINT, False))
        self.assertEqual(d1, wire.ble_setpoint(1, 0.5))
        self.assertEqual(d2, wire.ble_setpoint(2, 0, 0.2, 1.0))

    def test_arm_needs_a_setpoint_then_stop_disarms_and_notifications_arrive(self):
        peer = FakePeer()

        async def go():
            car = await ble_client.Car(peer).open()
            self.assertFalse(await car.arm(), "a rejected write is a refused arm")
            await car.send(0.1)
            self.assertTrue(await car.arm())
            self.assertEqual((await car.wait_status(lambda s: s.state == "Armed", 1)).state, "Armed")
            self.assertTrue(await car.stop())
            return await car.wait_status(lambda s: s.state == "Stopped", 1)
        self.assertEqual(run(go()), ("Stopped", "Stop", None))

    def test_wait_status_times_out(self):
        async def go():
            car = await ble_client.Car(FakePeer()).open()
            return await car.wait_status(lambda s: s.state == "Armed", 0.1)
        self.assertIsNone(run(go()))

    def test_stream_paces_at_20_hz(self):
        peer = FakePeer()

        async def go():
            car = await ble_client.Car(peer).open()
            return await car.stream(0.3, seconds=0.5)
        n = run(go())
        times = [t for u, _, _, t in peer.writes if u == ble_client.SETPOINT]
        self.assertEqual(n, len(times))
        self.assertIn(n, range(9, 12))
        self.assertAlmostEqual((times[-1] - times[0]) / (n - 1), 0.05, delta=0.01)

    def test_close_unsubscribes_and_disconnects(self):
        peer = FakePeer()

        async def go():
            car = await ble_client.Car(peer).open()
            await car.close()
        run(go())
        self.assertFalse(peer.is_connected)
        self.assertEqual(peer.notify, {})


class TimedPeer(FakePeer):
    """One central's link to a fake board with short windows: one owner at a time (the board does
    not advertise while owned), lease expiry, and the claim and reclaim windows, which end in a
    disconnect from the board. Only for running ble_scenarios.py's code, not a model of the firmware."""
    owner = None
    lease, claim, reclaim = 0.3, 0.6, 0.5

    async def connect(self):
        if TimedPeer.owner:
            raise ble_client.BleakError("not found: the car does not advertise while owned")
        TimedPeer.owner, self.claimed_at, self.active = self, time.monotonic(), False
        await super().connect()
        self.ticker = asyncio.create_task(self.tick())

    async def disconnect(self):
        if TimedPeer.owner is self:
            TimedPeer.owner = None
        if self.is_connected and asyncio.current_task() is not self.ticker:
            self.ticker.cancel()
        await super().disconnect()

    async def read_gatt_char(self, uuid):
        if uuid == ble_client.CAPABILITIES:
            ms = (int(self.lease * 1000), int(self.claim * 1000), int(self.reclaim * 1000))
            return bytearray(CAPS[:-6] + struct.pack("<3H", *ms))
        return await super().read_gatt_char(uuid)

    async def write_gatt_char(self, uuid, data, response=False):
        self.active = True
        await super().write_gatt_char(uuid, data, response)
        if uuid == ble_client.CONTROL and data == b"\x01":
            self.reclaim_from = None  # arming again ends the reclaim window

    async def tick(self):
        """The release rules: the claim window, lease expiry, and the reclaim window, which starts
        when the lease lapses (at once on expiry; after a STOP only once the owner goes quiet)."""
        self.reclaim_from = None
        while self.is_connected:
            await asyncio.sleep(0.01)
            now, (state, reason, _) = time.monotonic(), self.state
            lapsed = self.last_setpoint is None or now - self.last_setpoint > self.lease
            if state == 1 and not self.active and now - self.claimed_at > self.claim:
                await self.disconnect()
            elif state == 2 and lapsed:
                self.set([3, 2, 0])
                self.reclaim_from = now
            elif state == 3 and self.reclaim_from is None and lapsed:
                self.reclaim_from = now
            elif state == 3 and self.reclaim_from and now - self.reclaim_from > self.reclaim:
                await self.disconnect()


class Scenarios(unittest.TestCase):
    def test_scenarios_run_against_a_timed_fake_board(self):
        """Runs the board scenarios' own code (no radio, no car log); a board run is the evidence."""
        import ble_scenarios

        async def connect(addr=None, timeout=10.0, adapter=None):
            return await ble_client.Car(TimedPeer()).open()

        async def find(addr=None, timeout=10.0, adapter=None):
            end = time.monotonic() + timeout
            while TimedPeer.owner and time.monotonic() < end:
                await asyncio.sleep(0.01)
            return None if TimedPeer.owner else "car"

        class Fake(ble_scenarios.Scenarios):
            __unittest_skip__ = False

        saved = ble_client.connect, ble_client.find, ble_scenarios.ADAPTER2, ble_scenarios.CARLOG
        ble_client.connect, ble_client.find = connect, find
        ble_scenarios.ADAPTER2, ble_scenarios.CARLOG = "hci1", None
        try:
            with open(os.devnull, "w") as null:
                r = unittest.TextTestRunner(stream=null).run(unittest.defaultTestLoader.loadTestsFromTestCase(Fake))
        finally:
            ble_client.connect, ble_client.find, ble_scenarios.ADAPTER2, ble_scenarios.CARLOG = saved
        self.assertEqual((r.failures, r.errors, r.skipped), ([], [], []))
        self.assertGreaterEqual(r.testsRun, 10)

    def test_skip_with_a_clear_message_without_a_board(self):
        env = {k: v for k, v in os.environ.items() if k != "ROVI_BLE_ADDR"}
        here = pathlib.Path(__file__).resolve().parent
        r = subprocess.run([sys.executable, str(here / "ble_scenarios.py")], env=env, cwd=here,
                           capture_output=True, text=True, timeout=30)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("ROVI_BLE_ADDR", r.stderr)
        self.assertIn("skipped", r.stderr)


if __name__ == "__main__":
    unittest.main()
