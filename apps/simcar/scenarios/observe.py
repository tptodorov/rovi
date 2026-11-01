#!/usr/bin/env python3
"""Observational conformance (add-rovi-protocol task 3.3, design D6): the same rules against the
sim car or the real car, through the shared `rovi_protocol` types only.

Never arms and never sends a nonzero setpoint, so the device run is safe with motor power off.
Rules: INFO, Echo, malformed-input rejection, claim/BUSY/not-owner/release over UDP; BLE reads and
Echo agreeing with UDP.

    ROVI_TARGET=<host:port> ROVI_OBSERVE=sim    python3 observe.py   # starts nothing; sim car running
    ROVI_TARGET=<car ip>:7777 ROVI_OBSERVE=device python3 observe.py  # real car, Bleak for BLE

The sim target reaches its virtual BLE on the UDP port + 1 (apps/simcar/src/vble.rs); the device
target finds the car by name "Rovi" and checks the service UUID in its GATT table.
"""
import asyncio
import json
import os
import socket
import struct
import time
import unittest

import rovi_protocol as rp

HOST, PORT = os.environ["ROVI_TARGET"].rsplit(":", 1)
PORT = int(PORT)
TARGET = os.environ.get("ROVI_OBSERVE", "sim")
assert TARGET in ("sim", "device"), TARGET
WAIT = 0.5 if TARGET == "device" else 0.2  # Wi-Fi adds latency the loopback sim does not


class Udp:
    def __init__(self):
        self.s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.s.bind(("", 0))
        self.session, self.seq = 0, 100

    def close(self):
        self.s.close()

    def send(self, message, session=None):
        self.seq += 1
        frame = {"session": self.session if session is None else session, "seq": self.seq, "message": message}
        self.s.sendto(rp.udp_encode_to_car(frame), (HOST, PORT))

    def raw(self, data):
        self.s.sendto(data, (HOST, PORT))

    def reply(self, timeout=WAIT):
        """The car's reply to the last request, decoded by the crate; None when it stays silent."""
        end = time.monotonic() + timeout
        while (left := end - time.monotonic()) > 0:
            self.s.settimeout(left)
            try:
                d = self.s.recvfrom(1500)[0]
            except socket.timeout:
                return None
            f = rp.udp_decode_from_car(d)  # every car reply must be valid under the crate
            if f["seq"] == self.seq:
                self.reply_session = f["session"]
                return f["message"]
        return None

    def info(self):
        self.send({"kind": "info"}, session=0)
        m = self.reply()
        assert m and m["kind"] == "info", m
        return m


class SimBle:
    """The sim car's virtual BLE central (`rovi_protocol` ble::sim datagrams on port + 1)."""

    def __init__(self):
        self.s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.s.bind(("", 0))

    def _ask(self, req, op):
        self.s.sendto(req, (HOST, PORT + 1))
        self.s.settimeout(WAIT)
        while True:
            d = self.s.recvfrom(512)[0]
            if d[0] == op:
                return d[1:]

    async def connect(self):
        assert self._ask(b"\x01", 0x81) == b"\x01", "virtual BLE refused the connection"

    async def read(self, name):
        return self._ask(bytes([4, {"status": 1, "capabilities": 2, "network": 3}[name]]), 0x84)[1:]

    async def echo(self, data):
        return self._ask(b"\x05" + data, 0x85)

    async def disconnect(self):
        self._ask(b"\x06", 0x86)
        self.s.close()


class DeviceBle:
    """A real central over BlueZ (Bleak). Reads and Echo only; never writes Setpoint or Control."""

    async def connect(self):
        from bleak import BleakClient, BleakScanner
        dev = await BleakScanner.find_device_by_name("Rovi", timeout=20)
        assert dev, "Rovi is not advertising"
        self.c = BleakClient(dev)
        await self.c.connect()
        assert self.c.services.get_service(rp.BLE_UUIDS["service"]), "Rovi service UUID missing"
        self.echoes = asyncio.Queue()
        await self.c.start_notify(rp.BLE_UUIDS["echo"], lambda _, d: self.echoes.put_nowait(bytes(d)))

    async def read(self, name):
        return bytes(await self.c.read_gatt_char(rp.BLE_UUIDS[name]))

    async def echo(self, data):
        await self.c.write_gatt_char(rp.BLE_UUIDS["echo"], data, response=False)
        return await asyncio.wait_for(self.echoes.get(), 2)

    async def disconnect(self):
        await self.c.disconnect()


class UdpRules(unittest.TestCase):
    def setUp(self):
        self.a, self.b = Udp(), Udp()
        self.assertEqual(self.a.info()["status"]["state"], "open", "the car must be open (no owner) to start")

    def tearDown(self):
        for c in (self.a, self.b):
            if c.session:
                c.send({"kind": "bye"})
            c.close()

    def test_info_is_valid_and_meets_the_client_minimum(self):
        m = self.a.info()
        self.assertTrue(rp.meets_client_minimum(m["capabilities"]), m["capabilities"])
        self.assertIn(m["network"]["mode"], ("home", "fallback", "attempting"))

    def test_echo_returns_the_payload_and_drops_oversized(self):
        data = bytes(range(rp.ECHO_MAX))
        self.a.send({"kind": "echo", "data": data}, session=0)
        self.assertEqual(self.a.reply(), {"kind": "echo", "data": data})
        self.a.seq += 1  # oversized Echo: raw, since the crate will not encode it
        self.a.raw(b"RV\x01\x09" + struct.pack("<II", 0, self.a.seq) + bytes(rp.ECHO_MAX + 1))
        self.assertIsNone(self.a.reply())

    def test_malformed_frames_get_no_reply_and_do_not_claim(self):
        hello = rp.udp_encode_to_car({"session": 0, "seq": self.a.seq + 1, "message": {"kind": "hello"}})
        for data in (hello + b"\x01", b"RX" + hello[2:], hello[:2] + b"\x02" + hello[3:], hello[:11]):
            self.a.raw(data)
        self.a.seq += 1
        self.assertIsNone(self.a.reply())
        self.assertEqual(self.a.info()["status"]["state"], "open")

    def test_claim_busy_not_owner_and_release(self):
        self.a.send({"kind": "hello"})
        ack = self.a.reply()
        self.assertEqual(ack["kind"], "hello-ack", ack)
        self.a.session = self.a.reply_session
        self.assertEqual(ack["capabilities"], self.b.info()["capabilities"])
        self.assertEqual(self.b.info()["status"]["state"], "claimed")
        self.b.send({"kind": "hello"})
        self.assertEqual(self.b.reply(), {"kind": "busy"})
        self.b.send({"kind": "arm"}, session=self.a.session)  # right session, wrong peer
        self.assertIsNone(self.b.reply(), "a non-owner gets no reply")
        self.assertEqual(self.b.info()["status"]["state"], "claimed", "and nothing changed")
        self.a.send({"kind": "bye"})
        self.a.session = 0
        time.sleep(WAIT)
        self.assertEqual(self.b.info()["status"]["state"], "open")


class BleRules(unittest.TestCase):
    def test_reads_and_echo_agree_with_udp(self):
        asyncio.run(self.check())

    async def check(self):
        udp = Udp()
        try:
            info = udp.info()
        finally:
            udp.close()
        ble = DeviceBle() if TARGET == "device" else SimBle()
        await ble.connect()
        try:
            self.assertEqual(rp.capabilities_decode(await ble.read("capabilities")), info["capabilities"])
            self.assertEqual(rp.status_decode(await ble.read("status")), info["status"])
            self.assertEqual(rp.network_decode(await ble.read("network")), info["network"])
            data = b"rovi-observe"
            self.assertEqual(await ble.echo(data), data)
        finally:
            await ble.disconnect()


if __name__ == "__main__":
    print(json.dumps({"target": TARGET, "car": f"{HOST}:{PORT}"}))
    unittest.main(verbosity=2)
