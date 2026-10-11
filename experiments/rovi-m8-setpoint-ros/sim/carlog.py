#!/usr/bin/env python3
"""Reads the car's `ev=` log lines (src/events.rs, docs/LOGGING.md) from the sim car's stdout or a board's serial capture.

    python3 sim/carlog.py car.log    # prints a summary, exits 1 if the log is inconsistent
"""
import collections, re, sys

# Keep in sync with src/events.rs (test_carlog.py checks it).
CATALOG = {"boot", "claimed", "released", "stop", "state", "ble", "wifi", "ros", "error", "dropped"}
PANIC = re.compile(r"panicked|PANIC")
EV = re.compile(r"(?:^|\s)(ev=\S+.*)")
INT = re.compile(r"-?\d+$")


def value(v):
    """An integer, a comma list of integers, or the token as it is."""
    if INT.match(v):
        return int(v)
    parts = v.split(",")
    if len(parts) > 1 and all(map(INT.match, parts)):
        return [int(p) for p in parts]
    return v


def parse(line):
    """One event dict, or None for other lines and for lines cut short (a serial capture can start mid-line)."""
    m = EV.search(line)
    if not m:
        return None
    e = {k: value(v) for k, v in (kv.split("=", 1) for kv in m.group(1).split() if "=" in kv)}
    return e if "at_ms" in e else None  # at_ms is the last field, so its absence means a cut line


def read(path, offset=0):
    with open(path, "rb") as f:
        f.seek(offset)
        return f.read().decode(errors="replace")


def events(path, offset=0):
    return [e for e in map(parse, read(path, offset).splitlines()) if e]


def segments(text):
    """Events grouped by boot. Events before the first `ev=boot` (a capture started mid-run) form their own group."""
    out = [[]]
    for line in text.splitlines():
        e = parse(line)
        if e:
            if e["ev"] == "boot":
                out.append([])
            out[-1].append(e)
    return [s for s in out if s]


def check(path):
    """(problems, boots, counts): panics, unknown events, faults, lost events, and claim/release that do not alternate."""
    text = read(path)
    problems = [f"panic: {line.strip()}" for line in text.splitlines() if PANIC.search(line)]
    counts = collections.Counter()
    for i, seg in enumerate(segments(text), 1):
        claimed = False
        for e in seg:
            name, at = e["ev"], e["at_ms"]
            counts[name] += 1
            if name not in CATALOG:
                problems.append(f"boot {i}: unknown event {name!r} at {at} ms")
            elif name == "error":
                problems.append(f"boot {i}: fault {e.get('src')} at {at} ms")
            elif name == "dropped":
                problems.append(f"boot {i}: {e.get('n')} events lost (log ring full) at {at} ms")
            elif name == "claimed":
                if claimed:
                    problems.append(f"boot {i}: claimed twice without release at {at} ms")
                claimed = True
            elif name == "released":
                if not claimed:
                    problems.append(f"boot {i}: released while open at {at} ms")
                claimed = False
    return problems, counts["boot"], counts


if __name__ == "__main__":
    problems, boots, counts = check(sys.argv[1])
    print(f"{sum(counts.values())} events in {boots} boot(s): {dict(counts)}")
    for p in problems:
        print("PROBLEM", p)
    sys.exit(1 if problems else 0)
