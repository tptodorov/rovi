#!/usr/bin/env python3
"""Probe the car's Wi-Fi link with UDP echoes while another process holds a BLE session (Q20).

    python3 sim/ap_probe.py [--seconds N] [-- command ...]

Sends one echo every 200 ms to ROVI_TARGET (default 192.168.4.1:7777), prints a line per change
between reachable and unreachable with the seconds since the start, and, with a command after `--`,
runs it after 3 s. When the command ends it keeps probing for --settle seconds. If the link is
still down it runs `nmcli connection up rovi-m8` and reports how long the rejoin took.
"""
import argparse, os, socket, struct, subprocess, sys, threading, time

ECHO = 9

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seconds", type=float, default=30.0, help="probe at least this long")
    ap.add_argument("--settle", type=float, default=20.0, help="s to wait for the link after the command")
    ap.add_argument("--profile", default="rovi-m8", help="NetworkManager profile to bring up if the link stays down")
    args, rest = ap.parse_known_args()
    cmd = rest[1:] if rest[:1] == ["--"] else rest
    host, port = os.environ.get("ROVI_TARGET", "192.168.4.1:7777").rsplit(":", 1)
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.settimeout(0.15)
    t0 = time.monotonic()
    state = {"up": None, "done": None}
    log = []

    def note(msg):
        line = f"{time.monotonic() - t0:6.1f}s {msg}"
        log.append(line)
        print(line, flush=True)

    def run():
        state["rc"] = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode
        state["done"] = time.monotonic()
        note(f"command ended (exit {state['rc']})")

    started = False
    seq = 0
    rejoin_from = None
    while True:
        now = time.monotonic() - t0
        if cmd and not started and now >= 3:
            started = True
            note("command starts: " + " ".join(cmd)[:60])
            threading.Thread(target=run, daemon=True).start()
        seq += 1
        payload = b"AP" + struct.pack("<I", seq)
        ok = False
        try:
            s.sendto(b"RV\x01" + bytes([ECHO]) + struct.pack("<II", 0, seq) + payload, (host, int(port)))
            d, _ = s.recvfrom(256)
            ok = d[12:] == payload
        except OSError:
            pass
        if ok != state["up"]:
            note("link up" if ok else "link DOWN")
            state["up"] = ok
        end = state["done"] is not None and (ok or time.monotonic() - state["done"] > args.settle)
        if end and not ok and cmd:
            note(f"still down {args.settle:.0f}s after the command: rejoining with nmcli")
            rejoin_from = time.monotonic()
            r = subprocess.run(["nmcli", "--wait", "20", "connection", "up", args.profile], capture_output=True)
            note(f"nmcli exit {r.returncode}")
            end = False
            state["done"] = None
            args.seconds = 0
            while time.monotonic() - rejoin_from < 30:
                try:
                    s.sendto(b"RV\x01" + bytes([ECHO]) + struct.pack("<II", 0, 0) + b"AP\0\0\0\0", (host, int(port)))
                    s.recvfrom(256)
                    note(f"link up again {time.monotonic() - rejoin_from:.1f}s after nmcli started")
                    break
                except OSError:
                    time.sleep(0.2)
            break
        if (not cmd or end) and now >= args.seconds:
            break
        time.sleep(0.2)
    sys.exit(0)

if __name__ == "__main__":
    main()
