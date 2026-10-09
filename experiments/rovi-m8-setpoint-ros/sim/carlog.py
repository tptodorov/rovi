#!/usr/bin/env python3
"""Reads the car's `ev=` log lines (src/events.rs) from the sim car's stdout or a board's serial capture.

    python3 sim/carlog.py car.log    # prints a summary, exits 1 if the log is inconsistent
"""
import collections, re, sys

BOOT = ("simcar listening", "M8 board-only")  # first lines of the sim car and the firmware
PANIC = re.compile(r"panicked|PANIC")
EV = re.compile(r"(?:^|\s)(ev=\S+.*)")


def parse(line):
    """One event dict, or None for other lines and for lines cut short (serial capture starts mid-line)."""
    m = EV.search(line)
    if not m:
        return None
    e = dict(kv.split("=", 1) for kv in m.group(1).split() if "=" in kv)
    if "at_ms" not in e:  # at_ms is the last field, so its absence means a truncated line
        return None
    e["at_ms"] = int(e["at_ms"])
    if "wheels" in e:
        e["wheels"] = [float(w) for w in e["wheels"].split(",")]
    return e


def events(path, offset=0):
    with open(path, "rb") as f:
        f.seek(offset)
        text = f.read().decode(errors="replace")
    return [e for e in map(parse, text.splitlines()) if e]


def check(path):
    """Problems found in the log: panics, and claim/release that do not alternate within one boot."""
    problems, claimed, boots, counts = [], False, 0, collections.Counter()
    with open(path, errors="replace") as f:
        lines = f.read().splitlines()
    for n, line in enumerate(lines, 1):
        if any(b in line for b in BOOT):
            boots, claimed = boots + 1, False
        if PANIC.search(line):
            problems.append(f"line {n}: panic: {line.strip()}")
        e = parse(line)
        if not e:
            continue
        counts[e["ev"]] += 1
        if e["ev"] == "claimed":
            if claimed:
                problems.append(f"line {n}: claimed twice without release at {e['at_ms']} ms")
            claimed = True
        elif e["ev"] == "released":
            if not claimed:
                problems.append(f"line {n}: released while open at {e['at_ms']} ms")
            claimed = False
    return problems, boots, counts


if __name__ == "__main__":
    problems, boots, counts = check(sys.argv[1])
    print(f"{sum(counts.values())} events in {boots} boot(s): {dict(counts)}")
    for p in problems:
        print("PROBLEM", p)
    sys.exit(1 if problems else 0)
