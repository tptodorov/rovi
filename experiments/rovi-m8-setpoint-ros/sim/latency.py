#!/usr/bin/env python3
"""Round-trip latency over the car's echo (UDP `ECHO` frame, BLE Echo characteristic): sends N timestamped
probes at a fixed rate and prints sent, received count, loss, and median, p95 and max RTT in ms.

    (cd sim/simcar && cargo build) && python3 sim/latency.py      # a fresh sim car: development evidence only
    ROVI_TARGET=192.168.4.1:7777 python3 sim/latency.py           # a running car: join its AP first
    python3 sim/latency.py --ble <address>                        # BLE, needs `bleak`; connecting claims the car

The echo is stateless and open to any UDP peer, so the UDP run works while another client owns the car.
Over BLE the connection is the session, so the BLE run only connects while the car is free, and owns it meanwhile. It also streams zero setpoints at the probe rate: echoes are not activity, and the car drops an owner that sends no setpoint within the claim window.
"""
import argparse, asyncio, math, os, pathlib, select, socket, statistics, struct, subprocess, sys, time

import wire

ECHO = 9
ECHO_MAX = 32  # device::ECHO_MAX
MAGIC = b"RL"
PROBE = struct.Struct("<2sIQ")  # magic, probe number, client send time in ns
ECHO_UUID = "d47a0015-45b2-4d19-8db0-6bd87e9c0001"
SETPOINT_UUID = "d47a0011-45b2-4d19-8db0-6bd87e9c0001"
BIN = pathlib.Path(__file__).resolve().parent / "simcar/target/debug/simcar"
SIM_PORT = 17778


def probe(i, t_ns):
    return PROBE.pack(MAGIC, i, t_ns)


def unprobe(data):
    """(probe number, send time in ns), or None for a payload that is not one of ours."""
    if len(data) != PROBE.size or data[:2] != MAGIC:
        return None
    return PROBE.unpack(data)[1:]


def stats(rtts_ms, sent):
    r = sorted(rtts_ms)
    return {
        "sent": sent,
        "count": len(r),
        "loss_pct": round(100 * (sent - len(r)) / sent, 1) if sent else 0.0,
        "median_ms": statistics.median(r) if r else None,
        "p95_ms": r[math.ceil(0.95 * len(r)) - 1] if r else None,  # nearest rank
        "max_ms": r[-1] if r else None,
    }


def line(transport, s):
    ms = lambda v: "-" if v is None else f"{v:.3f}"
    return (f"{transport} sent={s['sent']} count={s['count']} loss_pct={s['loss_pct']} "
            f"median_ms={ms(s['median_ms'])} p95_ms={ms(s['p95_ms'])} max_ms={ms(s['max_ms'])}")


class Collector:
    """Matches echoed probes to RTTs; ignores duplicates and foreign payloads."""

    def __init__(self):
        self.rtts = {}

    def got(self, payload):
        p = unprobe(payload)
        if p and p[0] not in self.rtts:
            self.rtts[p[0]] = (time.perf_counter_ns() - p[1]) / 1e6

    def ms(self):
        return list(self.rtts.values())


def udp(host, port, n=200, hz=20.0, wait=1.0):
    """RTTs in ms of the echoes that came back within `wait` s of the last send."""
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    c, t0 = Collector(), time.perf_counter()
    try:
        for i in range(n + 1):
            due = t0 + i / hz if i < n else t0 + (n - 1) / hz + wait
            while (left := due - time.perf_counter()) > 0:
                if select.select([s], [], [], left)[0]:
                    d = s.recv(1500)
                    if d[:4] == b"RV\x01" + bytes([ECHO]):
                        c.got(d[12:])
            if i < n:
                s.sendto(b"RV\x01" + bytes([ECHO]) + struct.pack("<II", 0, i) + probe(i, time.perf_counter_ns()),
                         (host, port))
    finally:
        s.close()
    return c.ms()


async def ble(address, n=200, hz=20.0, wait=1.0):
    """The same over the BLE Echo characteristic (write without response, notify)."""
    try:
        from bleak import BleakClient, BleakScanner
    except ImportError:
        sys.exit("BLE needs bleak: pip install bleak")
    c = Collector()
    # Connecting is the claim, and the car releases an owner that sends no setpoint within the claim
    # window (10 s), then drops the link. Echo writes do not count as activity, so stream zero
    # setpoints alongside the probes, as a real controller would. Without that the first 10 s work
    # and every later write fails with "Service Discovery has not been performed yet" (seen on the board).
    device = await BleakScanner.find_device_by_address(address, timeout=15)
    if device is None:
        sys.exit(f"BLE device {address} not seen: is the car advertising (Open, LED off)?")
    async with BleakClient(device) as cl:
        await cl.start_notify(ECHO_UUID, lambda _, d: c.got(bytes(d)))
        t0 = time.perf_counter()
        for i in range(n):
            await asyncio.sleep(max(0.0, t0 + i / hz - time.perf_counter()))
            await cl.write_gatt_char(SETPOINT_UUID, wire.ble_setpoint(i + 1), response=False)
            await cl.write_gatt_char(ECHO_UUID, probe(i, time.perf_counter_ns()), response=False)
        await asyncio.sleep(wait)
    return c.ms()


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("-n", type=int, default=200, help="probes (default 200)")
    ap.add_argument("--hz", type=float, default=20.0, help="send rate (default 20)")
    ap.add_argument("--wait", type=float, default=1.0, help="s to wait for late echoes (default 1)")
    ap.add_argument("--ble", metavar="ADDRESS", help="use BLE to this address instead of UDP")
    a = ap.parse_args()
    if a.ble:
        print(line("ble", stats(asyncio.run(ble(a.ble, a.n, a.hz, a.wait)), a.n)))
        return
    target, car = os.environ.get("ROVI_TARGET"), None
    if target:
        host, port = target.rsplit(":", 1)[0], int(target.rsplit(":", 1)[1])
    else:
        host, port = "127.0.0.1", SIM_PORT
        car = subprocess.Popen([BIN, str(port)], stdout=subprocess.DEVNULL)
        time.sleep(0.2)
    try:
        print(line("udp", stats(udp(host, port, a.n, a.hz, a.wait), a.n)))
    finally:
        if car:
            car.terminate()
            car.wait()


if __name__ == "__main__":
    main()
