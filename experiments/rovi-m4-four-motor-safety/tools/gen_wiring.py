#!/usr/bin/env python3
"""Generate ../wiring.kicad_sch (M4: four motors, two TB6612FNG drivers). Edit this file, not the schematic.

Every connected pin gets a short stub wire with a net label at its outer end; nets join by label
name only. No wire crosses another symbol's stubs, so the label-on-wire merge trap of
docs/reference/kicad-authoring.md cannot happen by construction (still run the checklist there).

Board symbols (ESP32-S3 CORE board, TB6612FNG breakout) carry every physical header pin in the real
order, numbered as in docs/reference/ and M1 (L1.. / R1..). Same pins as WIRING.md.

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
NS = uuid.UUID("6f1d1c3e-0000-4000-8000-000000000004")
OUT = os.path.join(os.path.dirname(__file__), "..", "wiring.kicad_sch")


def uid(*parts):
    return str(uuid.uuid5(NS, "/".join(map(str, parts))))


def f(v):
    return f"{v:.2f}".rstrip("0").rstrip(".")


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
TWO = [("1", "1")], [("2", "2")]

LIBS = {
    "ESP32S3_CORE": (30.48, ESP_LEFT, ESP_RIGHT),
    "TB6612FNG_BOARD": (25.4, TB_LEFT, TB_RIGHT),
    "TWO_PIN": (7.62, *TWO),
}

# --- Parts: reference, lib, value, (x, y), {pin number: net}. Unlisted pins get a no-connect. -----

GND = "GND"
PARTS = [
    ("U1", "ESP32S3_CORE", "ESP32-S3-N16R8 CORE board (USB-C from the laptop)", (213.36, 50.8), {
        "L1": "3V3", "L2": "3V3", "L4": "STBY", "L5": "D1_AIN1", "L6": "D1_AIN2", "L7": "D1_PWMA",
        "L8": "D1_PWMB", "L9": "D1_BIN1", "L10": "D1_BIN2", "L12": "D2_PWMA", "L15": "D2_AIN1",
        "L16": "D2_AIN2", "L17": "D2_PWMB", "L18": "D2_BIN1", "L19": "D2_BIN2", "L22": GND}),
    ("U2", "TB6612FNG_BOARD", "TB6612FNG D1: left wheels (M1's driver)", (139.70, 50.8), {
        "L1": "VM", "L2": "3V3", "L3": GND, "L4": "D1_AO1", "L5": "D1_AO2", "L6": "D1_BO2",
        "L7": "D1_BO1", "L8": GND, "R1": "D1_PWMA", "R2": "D1_AIN2", "R3": "D1_AIN1", "R4": "STBY",
        "R5": "D1_BIN1", "R6": "D1_BIN2", "R7": "D1_PWMB", "R8": GND}),
    ("U3", "TB6612FNG_BOARD", "TB6612FNG D2: right wheels", (139.70, 91.44), {
        "L1": "VM", "L2": "3V3", "L3": GND, "L4": "D2_AO1", "L5": "D2_AO2", "L6": "D2_BO2",
        "L7": "D2_BO1", "L8": GND, "R1": "D2_PWMA", "R2": "D2_AIN2", "R3": "D2_AIN1", "R4": "STBY",
        "R5": "D2_BIN1", "R6": "D2_BIN2", "R7": "D2_PWMB", "R8": GND}),
    ("M1", "TWO_PIN", "TT motor front-left (D1 A, M1's motor)", (35.56, 50.8), {"1": "D1_AO1", "2": "D1_AO2"}),
    ("M2", "TWO_PIN", "TT motor rear-left (D1 B)", (35.56, 66.04), {"1": "D1_BO1", "2": "D1_BO2"}),
    ("M3", "TWO_PIN", "TT motor front-right (D2 A)", (35.56, 91.44), {"1": "D2_AO1", "2": "D2_AO2"}),
    ("M4", "TWO_PIN", "TT motor rear-right (D2 B)", (35.56, 106.68), {"1": "D2_BO1", "2": "D2_BO2"}),
    ("BT1", "TWO_PIN", "5 V power bank (motor supply; unplug = kill switch)", (35.56, 137.16), {"1": "VM", "2": GND}),
]


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


ENDS = {}


def claim(pt, owner):
    """Two stubs sharing a point would merge their nets (kicad-authoring.md): refuse to generate."""
    key = (round(pt[0], 3), round(pt[1], 3))
    assert key not in ENDS, f"{owner} touches {ENDS[key]} at {key}"
    ENDS[key] = owner


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
    used = set()
    for side, pins in (("L", left), ("R", right)):
        for i, (num, _) in enumerate(pins):
            tip_x = x0 - STUB if side == "L" else x0 + width + STUB
            y = y0 + i * PITCH  # sheet_y = instance_y - local_y
            net = nets.get(num)
            if net is None:
                claim((tip_x, y), f"{ref}.{num} nc")
                body.append(f'\t(no_connect (at {f(tip_x)} {f(y)}) (uuid "{uid(ref, "nc", num)}"))')
                continue
            used.add(num)
            end_x = tip_x - STUB if side == "L" else tip_x + STUB
            claim((tip_x, y), f"{ref}.{num}")
            claim((end_x, y), f"{ref}.{num} label")
            body.append(f"\t(wire (pts (xy {f(tip_x)} {f(y)}) (xy {f(end_x)} {f(y)}))"
                        f' (stroke (width 0) (type default)) (uuid "{uid(ref, "w", num)}"))')
            just = "right" if side == "L" else "left"
            body.append(f'\t(label "{net}" (at {f(end_x)} {f(y)} 0)'
                        f' (effects (font (size 1.1 1.1)) (justify {just} bottom)) (uuid "{uid(ref, "l", num)}"))')
    missing = set(nets) - used
    assert not missing, f"{ref}: nets on unknown pins {missing}"


NOTES = """Rovi M4 wiring: four TT motors, two TB6612FNG drivers, ESP32-S3, 5 V power bank. Same pins as WIRING.md.
Nets join by label name. M1's wiring is D1 channel A (GPIO4-7) and is kept; everything new is on the ESP32 left header.
One STBY line (GPIO4) for both drivers: low from boot and unless armed; the TB6612 pull-down holds it low in reset.
Skipped: GPIO18 (high glitch at power-up), GPIO3/46 (strapping). Firmware CHANNELS: D1 A=FL, D1 B=RL, D2 A=FR, D2 B=RR.
No current monitor, dead-man or bench supply (ADR-0014)."""


def main():
    body = []
    for part in PARTS:
        place(*part, body)
    lines = ["(kicad_sch", "\t(version 20231120)", '\t(generator "rovi-gen-wiring")', '\t(generator_version "9.0")',
             f'\t(uuid "{uid("sheet")}")', '\t(paper "A3")', "\t(lib_symbols"]
    for name in LIBS:
        lines += lib_symbol(name)
    lines.append("\t)")
    lines.append('\t(text "Rovi M4 wiring -- ESP32-S3 + 2x TB6612FNG + 4 motors + power bank"'
                 f' (at 40.64 25.4 0) (effects (font (size 2.5 2.5)) (justify left)) (uuid "{uid("title")}"))')
    note = NOTES.replace('"', "'").replace("\n", "\\n")
    lines.append(f'\t(text "{note}" (at 40.64 165.1 0) (effects (font (size 1.4 1.4)) (justify left top))'
                 f' (uuid "{uid("notes")}"))')
    lines += body
    lines += ['\t(sheet_instances (path "/" (page "1")))', ")"]
    with open(OUT, "w") as fh:
        fh.write("\n".join(lines) + "\n")


def check():
    """Exports KiCad's own netlist and compares every named net with PARTS. Exit 1 on a difference."""
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
        if not name.startswith(("unconnected", "Net-(")):
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
