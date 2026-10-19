#!/usr/bin/env python3
"""UDP lease timing at zero wheels; board runs require a separately announced go.

Join the AP and start serial capture, then set ROVI_TARGET explicitly:
    python3 firmware/bench/udp_lease_timing.py --carlog <live-car.log>
"""
import argparse
from pathlib import Path
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parent))
from udp_stop_checks import close, released
import carlog
import udp_scenarios as udp
from wire import Capabilities, cdr


def trial(path):
    client = udp.Client()
    try:
        hello = client.hello()
        assert hello and hello[0] == udp.ACK and hello[1] != 0, hello
        lease_ms = Capabilities.decode(hello[2]).lease_ms
        assert lease_ms > 0, lease_ms
        offset = path.stat().st_size
        # One setpoint only: successful ARM proves it was accepted. Both clocks
        # are on the laptop; transport/capture/polling delays enlarge the bound.
        started = time.monotonic_ns()
        client.send(udp.SETPOINT, cdr(0.0))
        assert client.status(udp.ARM) == (2, 0, 0), "zero setpoint did not arm"
        deadline = started + (lease_ms + 1050) * 1_000_000
        while True:
            events = carlog.events(path, offset)
            observed = time.monotonic_ns()
            assert not any(e.get("ev") in ("boot", "error", "dropped", "released")
                           for e in events), events
            assert all(e.get("wheels_pm", [0, 0, 0, 0]) == [0, 0, 0, 0]
                       for e in events), "nonzero wheel command"
            stops = [e for e in events if e.get("ev") == "stop"]
            if stops:
                assert stops[0].get("reason") == "LeaseExpired", stops
                assert any(carlog.ev("state", state="Armed")(e) for e in events), events
                upper_ms = (observed - started) / 1_000_000
                assert upper_ms <= lease_ms + 50, (
                    f"observed upper bound {upper_ms:.3f} ms exceeds lease+50 "
                    f"({lease_ms + 50} ms); includes network/serial/host delay"
                )
                # Receive only: an unsolicited owner Status also confirms expiry.
                assert client.recv() == (udp.STATUS, client.session, bytes([3, 2, 0]))
                return upper_ms, lease_ms
            assert observed < deadline, "missing fresh serial lease stop"
            time.sleep(0.002)
    finally:
        cleanup_offset = path.stat().st_size
        close(client)
        released(path, cleanup_offset)


def check(path, trials=5):
    assert udp.TARGET, "set ROVI_TARGET explicitly"
    assert path.is_file(), "provide the live serial capture"
    assert trials > 0, "at least one trial is required"
    results = []
    for n in range(trials):
        upper_ms, lease_ms = trial(path)
        results.append(upper_ms)
        print(f"Trial {n + 1}/{trials}: upper_bound_ms={upper_ms:.3f} "
              f"lease_ms={lease_ms} limit_ms={lease_ms + 50}; zero wheels; released",
              flush=True)
    print(f"UDP lease timing: passed; max_upper_bound_ms={max(results):.3f}", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--carlog", type=Path, required=True)
    parser.add_argument("--trials", type=int, default=5)
    args = parser.parse_args()
    check(args.carlog, args.trials)
