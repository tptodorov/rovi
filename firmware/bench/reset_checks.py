#!/usr/bin/env python3
"""Boot, reset and power-loss checks with motor power present; run only after an
announced test and Todor's go. Wheels never move except the one announced drive.

    nix-shell --run 'python3 firmware/bench/reset_checks.py --dir <new evidence dir>'

Each serial capture opens with a hard reset (espflash monitor's default), so every
log starts at boot. After each boot the car must stay Open with no claim, no
nonzero wheel command and no fault for 5 s; whether a wheel twitched is Todor's
observation.

1. Idle resets x2.
2. Reset while driving: UDP vx=0.1 for 3 s, then reset mid-stream.
3. Power loss: Todor unplugs the ESP32 USB for ~5 s, then reconnects it.
The laptop joins `rovi-m8` for step 2; the caller restores the prior Wi-Fi.
"""
import argparse
import os
from pathlib import Path
import signal
import subprocess
import sys
import threading
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "apps/simcar/scenarios"))
import carlog

PORT = Path("/dev/serial/by-id/usb-1a86_USB_Single_Serial_5C85037678-if00")
QUIET_SECONDS = 5


class Capture:
    """espflash monitor into <dir>/<name>.log; opening it resets the board."""

    def __init__(self, directory, name):
        self.path = directory / f"{name}.log"
        self.file = self.path.open("wb")
        self.proc = subprocess.Popen(
            ["espflash", "monitor", "--port", str(PORT), "--non-interactive",
             "--skip-update-check"], stdout=self.file, stderr=subprocess.STDOUT)
        found, seen = carlog.wait(self.path, 0, carlog.ev("boot"), timeout=15)
        assert found, (name, "no boot", seen[-3:])
        print(f"  {name}: booted", flush=True)

    def quiet(self, name):
        time.sleep(QUIET_SECONDS)
        bad = [e for e in carlog.events(self.path)
               if e.get("ev") in ("claimed", "error", "stop")
               or (e.get("ev") == "state" and (e.get("state") != "Open" or any(e["wheels_pm"])))]
        assert not bad, (name, bad)
        print(f"  {name}: Open, no claim/fault/wheel command for {QUIET_SECONDS} s", flush=True)

    def close(self):
        self.proc.send_signal(signal.SIGINT)
        self.proc.wait(timeout=5)
        self.file.close()


def drive_then_reset(directory):
    os.environ["ROVI_TARGET"] = "192.168.4.1:7777"
    import udp_scenarios as udp
    from wire import cdr

    capture = Capture(directory, "drive")
    for _ in range(6):  # the AP comes up a few seconds after boot
        if subprocess.run(["nmcli", "--wait", "15", "connection", "up", "rovi-m8"],
                          capture_output=True).returncode == 0:
            break
        time.sleep(2)
    else:
        raise AssertionError("could not join rovi-m8")
    client = udp.Client()
    assert client.hello()[0] == udp.ACK
    client.send(udp.SETPOINT, cdr(0.1))
    assert client.status(udp.ARM) == (2, 0, 0), "ARM refused"
    stop = threading.Event()

    def stream():  # keeps streaming through the reset, as a live controller would
        while not stop.is_set():
            client.send(udp.SETPOINT, cdr(0.1))
            time.sleep(0.05)

    streamer = threading.Thread(target=stream)
    streamer.start()
    try:
        print("Gentle forward vx=0.1 for 3 s, then reset while driving", flush=True)
        time.sleep(3)
        found, _ = carlog.wait(capture.path, 0, carlog.ev("state", state="Armed",
                                                           wheels_pm=[83, 83, 83, 83]))
        assert found, "drive did not start"
        capture.close()
        after = Capture(directory, "reset-while-driving")  # resets mid-stream
        time.sleep(1)
    finally:
        stop.set()
        streamer.join()
        client.s.close()
    after.quiet("reset-while-driving")
    after.close()


def check(directory):
    directory.mkdir(parents=True, exist_ok=False)
    assert PORT.exists(), "rover USB serial not connected"
    for n in (1, 2):
        print(f"{n}. Idle reset", flush=True)
        capture = Capture(directory, f"idle-reset-{n}")
        capture.quiet(f"idle-reset-{n}")
        capture.close()
    print("3. Reset while driving", flush=True)
    drive_then_reset(directory)
    print("4. Power loss: UNPLUG the ESP32 USB cable now", flush=True)
    end = time.monotonic() + 60
    while PORT.exists():
        assert time.monotonic() < end, "USB not unplugged within 60 s"
        time.sleep(0.2)
    print("  unplugged; wait ~5 s, then PLUG IT BACK IN", flush=True)
    end = time.monotonic() + 120
    while not PORT.exists():
        assert time.monotonic() < end, "USB not reconnected within 120 s"
        time.sleep(0.2)
    time.sleep(1)
    capture = Capture(directory, "power-restored")
    capture.quiet("power-restored")
    capture.close()
    print("Reset checks: serial passed; wheel observation required", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dir", type=Path, required=True)
    check(parser.parse_args().dir)
