#!/usr/bin/env python3
"""Generate ../wiring.kicad_sch (M4: four motors, two TB6612FNG drivers). Edit this file, not the schematic.

Every connection is a drawn wire. Each net gets its own vertical lane in the channel between two
columns of parts (motors | drivers | ESP32); each pin runs straight to its net's lane, and nets that
span both channels (3V3, GND) join through a rail along the bottom. Pin rows of neighbouring columns
are offset by half a grid step, so no two nets ever share a horizontal line, and every wire meets
another only end to end (junction dots where three meet). Board symbols carry every physical header
pin in the real order, numbered as in docs/reference/ and M1 (L1.. / R1..), same pins as WIRING.md.

  python3 tools/gen_wiring.py           # from experiments/rovi-m4-four-motor-safety/
  python3 tools/gen_wiring.py --check   # also: KiCad's netlist must equal PARTS, net by net
"""

import os
import re
import subprocess
import sys
import tempfile
import uuid

PITCH = 2.54
STUB = 5.08
NS = uuid.UUID("6f1d1c3e-0000-4000-8000-000000000005")
OUT = os.path.join(os.path.dirname(__file__), "..", "wiring.kicad_sch")


def uid(*parts):
    return str(uuid.uuid5(NS, "/".join(map(str, parts))))


def f(v):
    return f"{v:.2f}".rstrip("0").rstrip(".")


def key(x, y):
    return (round(x, 2), round(y, 2))


# --- Symbols: (lib name, width, left pins, right pins); a pin is (number, name). -------------------

ESP_LEFT = [("L1", "3V3"), ("L2", "3V3"), ("L3", "RST"), ("L4", "4"), ("L5", "5"), ("L6", "6"),
            ("L7", "7"), ("L8", "15"), ("L9", "16"), ("L10", "17"), ("L11", "18"), ("L12", "8"),
            ("L13", "3"), ("L14", "46"), ("L15", "9"), ("L16", "10"), ("L17", "11"), ("L18", "12"),
            ("L19", "13"), ("L20", "14"), ("L21", "5Vin"), ("L22", "GND")]
ESP_RIGHT = [("R1", "GND"), ("R2", "43"), ("R3", "44"), ("R4", "1"), ("R5", "2"), ("R6", "42"),
             ("R7", "41"), ("R8", "40"), ("R9", "39"), ("R10", "38"), ("R11", "37"), ("R12", "36"),
             ("R13", "35"), ("R14", "0"), ("R15", "45"), ("R16", "48"), ("R17", "47"), ("R18", "21"),
             ("R19", "20"), ("R20", "19"), ("R21", "GND"), ("R22", "GND")]
TB_LEFT = [("L1", "VM"), ("L2", "VCC"), ("L3", "GND"), ("L4", "AO1"), ("L5", "AO2"), ("L6", "BO2"),
           ("L7", "BO1"), ("L8", "GND")]
TB_RIGHT = [("R1", "PWMA"), ("R2", "AIN2"), ("R3", "AIN1"), ("R4", "STBY"), ("R5", "BIN1"),
            ("R6", "BIN2"), ("R7", "PWMB"), ("R8", "GND")]
MOTOR_PINS = [("1", "1"), ("2", "2")]
BANK_PINS = [("1", "+5V"), ("2", "-")]
CAP_POL = [("1", "+"), ("2", "-")]
CAP_PINS = [("1", "1"), ("2", "2")]
USB_PINS = [("1", "VBUS"), ("2", "GND")]

LIBS = {
    "ESP32S3_CORE": (30.48, ESP_LEFT, ESP_RIGHT),
    "TB6612FNG_BOARD": (25.4, TB_LEFT, TB_RIGHT),
    "MOTOR": (10.16, [], MOTOR_PINS),
    "POWER_BANK": (15.24, [], BANK_PINS),
    "POWER_BANK_MIRRORED": (15.24, BANK_PINS, []),
    "CAP_POLARIZED": (10.16, [], CAP_POL),
    "CAP": (10.16, [], CAP_PINS),
    # The ESP32 board's USB-C COM port; on the board it feeds 5Vin and GND (not drawn).
    "USB_C_ON_BOARD": (30.48, [], USB_PINS),
}

# --- Parts: reference, lib, value, (x, y), {pin number: net}. Unlisted pins get a no-connect. -----
# Columns: motors and power bank | drivers | ESP32. Drivers' rows sit on the 2.54 grid; every other
# part's rows are offset by 1.27, so rows never coincide across a channel. Each driver's control
# header R1-R7 takes 7 consecutive ESP32 left-header pins: one ribbon cable each, drawn nearly
# straight (driver 1 R1 sits beside ESP32 L4, driver 2 R1 beside L14).

GND = "GND"
PARTS = [
    ("U1", "ESP32S3_CORE", "ESP32-S3-N16R8 CORE board (powered by BT2 via USB-C)", (243.84, 39.37), {
        "L1": "3V3", "L2": "3V3", "L4": "D1_PWMA", "L5": "D1_AIN2", "L6": "D1_AIN1", "L7": "D1_STBY",
        "L8": "D1_BIN1", "L9": "D1_BIN2", "L10": "D1_PWMB", "L14": "D2_PWMA", "L15": "D2_AIN2",
        "L16": "D2_AIN1", "L17": "D2_STBY", "L18": "D2_BIN1", "L19": "D2_BIN2", "L20": "D2_PWMB",
        "L22": GND, "R1": GND}),
    ("U2", "TB6612FNG_BOARD", "D1: TB6612FNG, left wheels", (127.0, 45.72), {
        "L1": "VM", "L2": "3V3", "L3": GND, "L4": "D1_AO1", "L5": "D1_AO2", "L6": "D1_BO2",
        "L7": "D1_BO1", "R1": "D1_PWMA", "R2": "D1_AIN2", "R3": "D1_AIN1", "R4": "D1_STBY",
        "R5": "D1_BIN1", "R6": "D1_BIN2", "R7": "D1_PWMB"}),
    ("U3", "TB6612FNG_BOARD", "D2: TB6612FNG, right wheels", (127.0, 73.66), {
        "L1": "VM", "L2": "3V3", "L3": GND, "L4": "D2_AO1", "L5": "D2_AO2", "L6": "D2_BO2",
        "L7": "D2_BO1", "R1": "D2_PWMA", "R2": "D2_AIN2", "R3": "D2_AIN1", "R4": "D2_STBY",
        "R5": "D2_BIN1", "R6": "D2_BIN2", "R7": "D2_PWMB"}),
    ("M1", "MOTOR", "Motor front-left (D1 A)", (25.4, 39.37), {"1": "D1_AO1", "2": "D1_AO2"}),
    ("C3", "CAP", "100 nF ceramic across M1", (25.4, 52.07), {"1": "D1_AO1", "2": "D1_AO2"}),
    ("M2", "MOTOR", "Motor rear-left (D1 B)", (25.4, 64.77), {"1": "D1_BO1", "2": "D1_BO2"}),
    ("C4", "CAP", "100 nF ceramic across M2", (25.4, 77.47), {"1": "D1_BO1", "2": "D1_BO2"}),
    ("M3", "MOTOR", "Motor front-right (D2 A)", (25.4, 90.17), {"1": "D2_AO1", "2": "D2_AO2"}),
    ("C5", "CAP", "100 nF ceramic across M3", (25.4, 102.87), {"1": "D2_AO1", "2": "D2_AO2"}),
    ("M4", "MOTOR", "Motor rear-right (D2 B)", (25.4, 115.57), {"1": "D2_BO1", "2": "D2_BO2"}),
    ("C6", "CAP", "100 nF ceramic across M4", (25.4, 128.27), {"1": "D2_BO1", "2": "D2_BO2"}),
    ("C1", "CAP_POLARIZED", "220-470 uF >= 10 V at D1 VM", (25.4, 140.97), {"1": "VM", "2": GND}),
    ("C2", "CAP_POLARIZED", "220-470 uF >= 10 V at D2 VM", (25.4, 153.67), {"1": "VM", "2": GND}),
    ("BT1", "POWER_BANK", "Motor power bank, 5 V USB (unplug = kill switch)", (20.32, 166.37), {"1": "VM", "2": GND}),
    ("J1", "USB_C_ON_BOARD", "ESP32 board USB-C (COM port)", (243.84, 107.95), {"1": "ESP_5V", "2": GND}),
    ("BT2", "POWER_BANK_MIRRORED", "ESP32 power bank, 5 V USB", (330.2, 113.03), {"1": "ESP_5V", "2": GND}),
]

# Channels by x: where a pin tip lies decides which channel its wire runs in.
CHANNELS = {"A": (40, 122), "B": (157, 239), "C": (279, 326)}
# Lanes: x of each net's vertical in each channel; nets in more than one channel join on a rail.
LANES = {
    "A": {"D1_AO1": 55.88, "D1_AO2": 60.96, "D1_BO2": 66.04, "D1_BO1": 71.12, "D2_AO1": 76.2,
          "D2_AO2": 81.28, "D2_BO2": 86.36, "D2_BO1": 91.44, "VM": 101.6, "3V3": 106.68, GND: 111.76},
    "B": {"D1_PWMA": 167.64, "D1_AIN2": 171.45, "D1_AIN1": 175.26, "D1_STBY": 179.07,
          "D1_BIN1": 182.88, "D1_BIN2": 186.69, "D1_PWMB": 190.5, "D2_PWMA": 196.85,
          "D2_AIN2": 200.66, "D2_AIN1": 204.47, "D2_STBY": 208.28, "D2_BIN1": 212.09,
          "D2_BIN2": 215.9, "D2_PWMB": 219.71, "3V3": 226.06, GND: 231.14},
    "C": {GND: 289.56, "ESP_5V": 297.18},
}
RAILS = {"3V3": 180.34, GND: 186.69}


def lib_symbol(name):
    width, left, right = LIBS[name]
    rows = max(len(left), len(right))
    out = [f'\t\t(symbol "M4:{name}"', "\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)",
           '\t\t\t(property "Reference" "U" (at 0 5 0) (effects (font (size 1.27 1.27)) hide))',
           f'\t\t\t(property "Value" "{name}" (at 0 7 0) (effects (font (size 1.27 1.27)) hide))',
           f'\t\t\t(symbol "{name}_1_1"',
           f"\t\t\t\t(rectangle (start 0 {f(PITCH)}) (end {f(width)} {f(-rows * PITCH)})"
           " (stroke (width 0.254) (type default)) (fill (type none)))"]
    for side, pins in (("L", left), ("R", right)):
        for i, (num, pname) in enumerate(pins):
            x, ang = (-STUB, 0) if side == "L" else (width + STUB, 180)
            out.append(f'\t\t\t\t(pin passive line (at {f(x)} {f(-i * PITCH)} {ang}) (length {f(STUB)})'
                       f' (name "{pname}" (effects (font (size 1.1 1.1))))'
                       f' (number "{num}" (effects (font (size 1.1 1.1)))))')
    out += ["\t\t\t)", "\t\t)"]
    return out


def pin_tips():
    """(ref, pin) -> (x, y, channel) for every connected pin; channel is where its wire runs."""
    tips = {}
    for ref, lib, _, (x0, y0), nets in PARTS:
        width, left, right = LIBS[lib]
        for side, pins in (("L", left), ("R", right)):
            for i, (num, _) in enumerate(pins):
                if num not in nets:
                    continue
                x = x0 - STUB if side == "L" else x0 + width + STUB
                y = y0 + i * PITCH
                ch = next(c for c, (lo, hi) in CHANNELS.items() if lo <= x <= hi)
                tips[(ref, num)] = (x, y, ch)
    return tips


def routes():
    """Wire segments per net, split at every tap so wires only ever meet end to end."""
    tips = pin_tips()
    by_net = {}
    for ref, _, _, _, nets in PARTS:
        for num, net in nets.items():
            by_net.setdefault(net, []).append(tips[(ref, num)])
    segs = []  # (net, x1, y1, x2, y2)
    for net, pins in by_net.items():
        lane_taps = {}
        for x, y, ch in pins:
            lx = LANES[ch][net]
            segs.append((net, x, y, lx, y))
            lane_taps.setdefault(lx, set()).add(y)
        rail = RAILS.get(net)
        rail_taps = set()
        for lx, ys in lane_taps.items():
            if rail is not None:
                ys = ys | {rail}
                rail_taps.add(lx)
            ys = sorted(ys)
            segs += [(net, lx, a, lx, b) for a, b in zip(ys, ys[1:])]
        xs = sorted(rail_taps)
        segs += [(net, a, rail, b, rail) for a, b in zip(xs, xs[1:])]
    return segs


def check_geometry(segs):
    """No wire end may touch another net's wire, and no two nets may run on the same line."""
    for i, (n1, ax, ay, bx, by) in enumerate(segs):
        for n2, cx, cy, dx, dy in segs[i + 1:]:
            if n1 == n2:
                continue
            for px, py in ((cx, cy), (dx, dy)):
                assert not on_seg(px, py, ax, ay, bx, by), f"{n2} end touches {n1} at {px},{py}"
            for px, py in ((ax, ay), (bx, by)):
                assert not on_seg(px, py, cx, cy, dx, dy), f"{n1} end touches {n2} at {px},{py}"
            if ay == by == cy == dy:
                assert max(ax, bx) <= min(cx, dx) or max(cx, dx) <= min(ax, bx), f"{n1}/{n2} overlap at y {ay}"
            if ax == bx == cx == dx:
                assert max(ay, by) <= min(cy, dy) or max(cy, dy) <= min(ay, by), f"{n1}/{n2} overlap at x {ax}"


def on_seg(px, py, ax, ay, bx, by):
    eps = 0.001
    if abs(ax - bx) < eps:
        return abs(px - ax) < eps and min(ay, by) - eps <= py <= max(ay, by) + eps
    return abs(py - ay) < eps and min(ax, bx) - eps <= px <= max(ax, bx) + eps


def place(ref, lib, value, at, nets, body):
    width, left, right = LIBS[lib]
    x0, y0 = at
    body.append(f'\t(symbol (lib_id "M4:{lib}") (at {f(x0)} {f(y0)} 0) (unit 1) (in_bom yes) (on_board yes)'
                f' (uuid "{uid(ref)}")')
    body.append(f'\t\t(property "Reference" "{ref}" (at {f(x0)} {f(y0 - 5.08)} 0)'
                " (effects (font (size 1.4 1.4)) (justify left)))")
    body.append(f'\t\t(property "Value" "{value}" (at {f(x0)} {f(y0 - 3.3)} 0)'
                " (effects (font (size 1.1 1.1)) (justify left)))")
    for num, _ in left + right:
        body.append(f'\t\t(pin "{num}" (uuid "{uid(ref, "pin", num)}"))')
    body.append(f'\t\t(instances (project "wiring" (path "/{uid("sheet")}" (reference "{ref}") (unit 1))))')
    body.append("\t)")
    unknown = set(nets) - {n for n, _ in left + right}
    assert not unknown, f"{ref}: nets on unknown pins {unknown}"
    for side, pins in (("L", left), ("R", right)):
        for i, (num, _) in enumerate(pins):
            if num not in nets:
                tip_x = x0 - STUB if side == "L" else x0 + width + STUB
                body.append(f'\t(no_connect (at {f(tip_x)} {f(y0 + i * PITCH)}) (uuid "{uid(ref, "nc", num)}"))')


NOTES = """Rovi M4 wiring: four TT motors, two TB6612FNG drivers, ESP32-S3, two 5 V power banks. Same pins as WIRING.md.
Ribbon cables: ESP32 left header L4-L10 (GPIO 4,5,6,7,15,16,17) -> D1 R1-R7; L14-L20 (GPIO 46,9,10,11,12,13,14) -> D2 R1-R7.
Each driver has its own STBY (GPIO7, GPIO11), low from boot and unless armed; the TB6612 pull-down holds it low in reset.
Firmware CHANNELS: D1 A=FL, D1 B=RL, D2 A=FR, D2 B=RR. Skipped: L11 GPIO18 (high glitch at power-up), L13 GPIO3 (strapping).
Power: two 5 V USB power banks. BT1 feeds both drivers' VM (motors); BT2 feeds the ESP32 through its USB-C port (J1).
C1/C2 bulk electrolytics at each driver's VM (mind polarity), C3-C6 100 nF ceramics soldered across each motor.
Grounds are common (ESP32 GND to both drivers' GND); 3V3 and GND join along the two rails at the bottom. No current monitor, dead-man or bench supply (ADR-0014)."""


def main():
    segs = routes()
    check_geometry(segs)
    body = []
    for part in PARTS:
        place(*part, body)
    ends = {}
    for net, ax, ay, bx, by in segs:
        for p in (key(ax, ay), key(bx, by)):
            ends[p] = ends.get(p, 0) + 1
        body.append(f"\t(wire (pts (xy {f(ax)} {f(ay)}) (xy {f(bx)} {f(by)}))"
                    f' (stroke (width 0) (type default)) (uuid "{uid("w", net, ax, ay, bx, by)}"))')
    for (x, y), n in sorted(ends.items()):
        if n >= 3:
            body.append(f'\t(junction (at {f(x)} {f(y)}) (diameter 0) (color 0 0 0 0) (uuid "{uid("j", x, y)}"))')
    # One label per net, on its first pin's wire, a stub away from the pin, so the netlist has names.
    named = set()
    for net, ax, ay, bx, by in segs:
        if net in named or ay != by:
            continue
        named.add(net)
        lx = ax + (STUB if bx > ax else -STUB)
        just = "left" if bx > ax else "right"
        body.append(f'\t(label "{net}" (at {f(lx)} {f(ay)} 0)'
                    f' (effects (font (size 1.0 1.0)) (justify {just} bottom)) (uuid "{uid("l", net)}"))')
    lines = ["(kicad_sch", "\t(version 20231120)", '\t(generator "rovi-gen-wiring")', '\t(generator_version "9.0")',
             f'\t(uuid "{uid("sheet")}")', '\t(paper "A3")', "\t(lib_symbols"]
    for name in LIBS:
        lines += lib_symbol(name)
    lines.append("\t)")
    lines.append('\t(text "Rovi M4 wiring -- ESP32-S3 + 2x TB6612FNG + 4 motors + 2 power banks"'
                 f' (at 25.4 22.86 0) (effects (font (size 2.5 2.5)) (justify left)) (uuid "{uid("title")}"))')
    note = NOTES.replace('"', "'").replace("\n", "\\n")
    lines.append(f'\t(text "{note}" (at 25.4 198.12 0) (effects (font (size 1.4 1.4)) (justify left top))'
                 f' (uuid "{uid("notes")}"))')
    lines += body
    lines += ['\t(sheet_instances (path "/" (page "1")))', ")"]
    with open(OUT, "w") as fh:
        fh.write("\n".join(lines) + "\n")


def check():
    """Exports KiCad's own netlist and compares every net with PARTS. Exit 1 on a difference."""
    want = {}
    for ref, _, _, _, nets in PARTS:
        for pin, net in nets.items():
            want.setdefault(net, set()).add((ref, pin))
    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, "x.net")
        subprocess.run(["kicad-cli", "sch", "export", "netlist", OUT, "-o", path], check=True,
                       capture_output=True)
        text = open(path).read()
    got = {}
    for chunk in text[text.index("(nets"):].split("(net\n")[1:]:
        name = re.search(r'\(name "([^"]+)"\)', chunk).group(1).lstrip("/")
        if not name.startswith("unconnected"):
            got[name] = set(re.findall(r'\(ref "([^"]+)"\)\s*\(pin "([^"]+)"\)', chunk))
    bad = sorted(n for n in set(want) | set(got) if want.get(n) != got.get(n))
    for n in bad:
        print(f"net {n}: intended {sorted(want.get(n, ()))}, netlist {sorted(got.get(n, ()))}")
    print(f"{len(want)} nets, {len(bad)} differ")
    return not bad


if __name__ == "__main__":
    main()
    if "--check" in sys.argv and not check():
        sys.exit(1)
