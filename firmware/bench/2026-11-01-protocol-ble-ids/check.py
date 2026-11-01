"""Read-only check of the #128 firmware: BLE by service UUID, reads only; UDP INFO and a malformed
HELLO. Never writes a BLE characteristic, never sends ARM or setpoints."""
import asyncio, json, socket, struct, sys, time
from bleak import BleakClient, BleakScanner

SERVICE = "d47a0010-45b2-4d19-8db0-6bd87e9c0001"
CHARS = {n: f"d47a00{c:02x}-45b2-4d19-8db0-6bd87e9c0001" for n, c in
         [("setpoint", 0x11), ("control", 0x12), ("capabilities", 0x13), ("status", 0x14), ("echo", 0x15), ("network", 0x16)]}
STATES = ["open", "claimed", "armed", "stopped"]


def status(b):
    assert len(b) == 3 and b[0] < 4 and b[1] < 5 and b[2] < 5, b.hex()
    assert (b[0] == 3) == (b[1] != 0), f"inconsistent {b.hex()}"
    return STATES[b[0]], b[1], b[2]


def network(b):
    assert len(b) == 8 and b[0] < 3 and b[1] < 3 and b[2] < 8 and b[3] == 0, b.hex()
    return {"mode": ["fallback", "attempting", "home"][b[0]], "pending": b[1], "reason": b[2], "ip": ".".join(map(str, b[4:]))}


def caps(b):
    assert len(b) == 27, len(b)
    c = struct.unpack("<B5f3H", bytes(b))
    assert c[0] == 1 and c[1] > 0 and c[2] > 0 and min(c[3:6]) >= 0 and min(c[6:]) > 0, c
    return c


async def ble():
    # The car advertises flags and its name only; the UUIDs are checked in the GATT table.
    dev = await BleakScanner.find_device_by_name("Rovi M8", timeout=20)
    assert dev, "Rovi M8 not advertising"
    async with BleakClient(dev) as c:
        svc = c.services.get_service(SERVICE)
        assert svc, "service UUID not in GATT table"
        found = {ch.uuid.lower() for ch in svc.characteristics}
        missing = [n for n, u in CHARS.items() if u not in found]
        assert not missing, f"missing characteristics {missing}"
        out = {"address": dev.address, "characteristics": sorted(found),
               "capabilities": caps(await c.read_gatt_char(CHARS["capabilities"])),
               "status": status(await c.read_gatt_char(CHARS["status"])),
               "network": network(await c.read_gatt_char(CHARS["network"]))}
    return out


def udp(ip):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.settimeout(1.0)
    s.sendto(b"RV\x01\x0b" + struct.pack("<II", 0, 41), (ip, 7777))
    p, _ = s.recvfrom(256)
    assert p[:4] == b"RV\x01\x0b" and len(p) == 12 + 38, p.hex()
    info = {"capabilities": caps(p[12:39]), "status": status(p[39:42]), "network": network(p[42:50])}
    # A HELLO with one stray payload byte: #128 discards it (no reply, no claim).
    s.sendto(b"RV\x01\x01" + struct.pack("<II", 0, 42) + b"\x01", (ip, 7777))
    try:
        r = s.recvfrom(256)[0]
        hello = f"UNEXPECTED reply {r.hex()}"
    except socket.timeout:
        hello = "no reply"
    s.sendto(b"RV\x01\x0b" + struct.pack("<II", 0, 43), (ip, 7777))
    after = status(s.recvfrom(256)[0][39:42])
    return info, hello, after


async def main():
    b = await ble()
    print("BLE", json.dumps(b, default=str))
    ip = b["network"]["ip"]
    info, hello, after = udp(ip)
    print("UDP INFO", json.dumps(info, default=str))
    print("malformed HELLO:", hello, "| state after:", after)
    assert info["capabilities"] == b["capabilities"], "capabilities differ BLE vs UDP"
    assert info["status"] == b["status"] and info["network"] == b["network"], "status/network differ BLE vs UDP"
    assert hello == "no reply" and after[0] == "open", "malformed HELLO was not discarded"
    print("PASS")


asyncio.run(main())
