#!/usr/bin/env python3
"""Generate ../wiring.kicad_sch (M4 integrated bench setup). Edit this file, not the schematic.

Every connected pin gets a short stub wire with a net label at its outer end; nets join by label
name only. No wire crosses another symbol's stubs, so the label-on-wire merge trap of
docs/reference/kicad-authoring.md cannot happen by construction (still run the checklist there).

Board symbols (ESP32-S3 CORE board, TB6612FNG breakout) carry every physical header pin in the real
order, numbered as in docs/reference/ and M1 (L1.. / R1..). Parts not yet bought (INA226 module,
logic analyzer) are numbered by function, not by a header: re-check them when they arrive.

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
# 74HC123 DIP-16, pin numbers from the Nexperia 74HC_HCT123 datasheet (Rev. 13), same as TI CD74HC123E.
# "~" marks the active-LOW output (overbar on the datasheet).
HC_LEFT = [("1", "1A"), ("2", "1B"), ("3", "~1RD"), ("4", "~1Q"), ("5", "2Q"), ("6", "2CEXT"),
           ("7", "2REXT/CEXT"), ("8", "GND")]
HC_RIGHT = [("16", "VCC"), ("15", "1REXT/CEXT"), ("14", "1CEXT"), ("13", "1Q"), ("12", "~2Q"),
            ("11", "~2RD"), ("10", "2B"), ("9", "2A")]
INA_LEFT = [("1", "VCC"), ("2", "GND"), ("3", "SCL"), ("4", "SDA"), ("5", "ALERT")]
INA_RIGHT = [("6", "IN+"), ("7", "IN-"), ("8", "VBUS")]
LA_LEFT = [("1", "CH0"), ("2", "CH1"), ("3", "CH2"), ("4", "CH3"), ("5", "CH4"), ("6", "CH5"),
           ("7", "CH6"), ("8", "CH7"), ("9", "GND")]
TWO = [("1", "1")], [("2", "2")]

LIBS = {
    "ESP32S3_CORE": (30.48, ESP_LEFT, ESP_RIGHT),
    "TB6612FNG_BOARD": (25.4, TB_LEFT, TB_RIGHT),
    "74HC123_DIP16": (25.4, HC_LEFT, HC_RIGHT),
    "INA226_MODULE": (20.32, INA_LEFT, INA_RIGHT),
    "LOGIC_ANALYZER": (17.78, LA_LEFT, []),
    "TWO_PIN": (7.62, *TWO),
}

# --- Parts: reference, lib, value, (x, y), {pin number: net}. Unlisted pins get a no-connect. -----

GND = "GND"
PARTS = [
    ("U1", "ESP32S3_CORE", "ESP32-S3-N16R8 CORE board", (213.36, 50.8), {
        "L1": "3V3", "L2": "3V3", "L3": "ESP_RST", "L4": "STBY_EN", "L5": "D1_AIN1", "L6": "D1_AIN2",
        "L7": "D1_PWMA", "L8": "D1_PWMB", "L9": "D1_BIN1", "L10": "D1_BIN2", "L12": "D2_PWMA",
        "L15": "D2_AIN1", "L16": "D2_AIN2", "L17": "D2_PWMB", "L18": "D2_BIN1", "L19": "D2_BIN2",
        "L20": "HEARTBEAT", "L22": GND, "R1": GND, "R4": "SDA", "R5": "SCL", "R9": "STBY",
        "R10": "INA_ALERT", "R21": GND, "R22": GND}),
    ("U2", "TB6612FNG_BOARD", "TB6612FNG D1 (FL, RL placeholder)", (139.70, 50.8), {
        "L1": "VM", "L2": "3V3", "L3": GND, "L4": "D1_AO1", "L5": "D1_AO2", "L6": "D1_BO2",
        "L7": "D1_BO1", "L8": GND, "R1": "D1_PWMA", "R2": "D1_AIN2", "R3": "D1_AIN1", "R4": "STBY",
        "R5": "D1_BIN1", "R6": "D1_BIN2", "R7": "D1_PWMB", "R8": GND}),
    ("U3", "TB6612FNG_BOARD", "TB6612FNG D2 (FR, RR placeholder)", (139.70, 91.44), {
        "L1": "VM", "L2": "3V3", "L3": GND, "L4": "D2_AO1", "L5": "D2_AO2", "L6": "D2_BO2",
        "L7": "D2_BO1", "L8": GND, "R1": "D2_PWMA", "R2": "D2_AIN2", "R3": "D2_AIN1", "R4": "STBY",
        "R5": "D2_BIN1", "R6": "D2_BIN2", "R7": "D2_PWMB", "R8": GND}),
    ("U4", "74HC123_DIP16", "74HC123 dead-man (CD74HC123E)", (312.42, 50.8), {
        "1": GND, "2": "HEARTBEAT", "3": "STBY_EN", "8": GND, "16": "3V3", "15": "RC1",
        "14": "CX1", "13": "STBY", "11": GND, "10": GND, "9": GND}),
    ("U5", "INA226_MODULE", "INA226 module (not bought)", (312.42, 93.98), {
        "1": "3V3", "2": GND, "3": "SCL", "4": "SDA", "5": "INA_ALERT", "6": "VSW", "7": "VM",
        "8": "VM"}),
    ("J1", "LOGIC_ANALYZER", "8ch 24MHz logic analyzer, capture A", (312.42, 129.54), {
        "1": "STBY_EN", "2": "HEARTBEAT", "3": "STBY", "4": "D1_PWMA", "5": "D1_AIN1", "6": "D1_AIN2",
        "7": "INA_ALERT", "8": "ESP_RST", "9": GND}),
    # Motor side.
    ("M1", "TWO_PIN", "TT motor on D1A (FL?)", (35.56, 50.8), {"1": "D1_AO1", "2": "D1_AO2"}),
    ("M2", "TWO_PIN", "TT motor on D1B (RL?)", (35.56, 66.04), {"1": "D1_BO1", "2": "D1_BO2"}),
    ("M3", "TWO_PIN", "TT motor on D2A (FR?)", (35.56, 91.44), {"1": "D2_AO1", "2": "D2_AO2"}),
    ("M4", "TWO_PIN", "TT motor on D2B (RR?)", (35.56, 106.68), {"1": "D2_BO1", "2": "D2_BO2"}),
    ("C3", "TWO_PIN", "100nF at M1 terminals", (86.36, 50.8), {"1": "D1_AO1", "2": "D1_AO2"}),
    ("C4", "TWO_PIN", "100nF at M2 terminals", (86.36, 66.04), {"1": "D1_BO1", "2": "D1_BO2"}),
    ("C5", "TWO_PIN", "100nF at M3 terminals", (86.36, 91.44), {"1": "D2_AO1", "2": "D2_AO2"}),
    ("C6", "TWO_PIN", "100nF at M4 terminals", (86.36, 106.68), {"1": "D2_BO1", "2": "D2_BO2"}),
    # Supply chain: PSU -> fuse -> kill switch -> INA226 shunt -> VM.
    ("PS1", "TWO_PIN", "Bench PSU, CV/CC, current-limited", (30.48, 167.64), {"1": "VSUP", "2": GND}),
    ("F1", "TWO_PIN", "Fuse (value: placeholder)", (71.12, 167.64), {"1": "VSUP", "2": "VFUSE"}),
    ("SW1", "TWO_PIN", "Kill switch on VM", (111.76, 167.64), {"1": "VFUSE", "2": "VSW"}),
    ("C1", "TWO_PIN", "Bulk cap at D1 VM (placeholder)", (152.40, 167.64), {"1": "VM", "2": GND}),
    ("C2", "TWO_PIN", "Bulk cap at D2 VM (placeholder)", (193.04, 167.64), {"1": "VM", "2": GND}),
    # Fail-safe passives.
    ("R1", "TWO_PIN", "10k STBY_EN pull-down", (30.48, 187.96), {"1": "STBY_EN", "2": GND}),
    ("R2", "TWO_PIN", "10k HEARTBEAT pull-down", (71.12, 187.96), {"1": "HEARTBEAT", "2": GND}),
    ("R3", "TWO_PIN", "10k STBY pull-down", (111.76, 187.96), {"1": "STBY", "2": GND}),
    ("R4", "TWO_PIN", "Rext 220k (placeholder)", (152.40, 187.96), {"1": "3V3", "2": "RC1"}),
    ("C7", "TWO_PIN", "Cext 1uF ceramic (placeholder)", (193.04, 187.96), {"1": "CX1", "2": "RC1"}),
    ("C8", "TWO_PIN", "100nF U4 decoupling", (233.68, 187.96), {"1": "3V3", "2": GND}),
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


NOTES = """Rovi M4 integrated bench setup -- DRAFT, nothing built or run. Nets join by label name; see WIRING.md.
Fail-safe (recommended option C): STBY = 74HC123 1Q. It is HIGH only while STBY_EN (GPIO4, on ~1RD) is HIGH
AND the control tick keeps toggling HEARTBEAT (GPIO14, on 1B) faster than tw = K*Rext*Cext (placeholder ~110 ms).
A hang with HEARTBEAT stuck high or low, a reset, or an unpowered ESP32 all leave STBY LOW (R1-R3 pull-downs).
Assumptions to verify: INA226 module pins/shunt (not bought; numbered by function); 74HC123 DIP pinout from the
Nexperia datasheet; TB6612 boards' on-board caps unknown; VBUS of the module may be tied to IN- internally.
Never probe AO/BO or VM with the logic analyzer (5 V max inputs, motor spikes)."""


def main():
    body = []
    for part in PARTS:
        place(*part, body)
    lines = ["(kicad_sch", "\t(version 20231120)", '\t(generator "rovi-gen-wiring")', '\t(generator_version "9.0")',
             f'\t(uuid "{uid("sheet")}")', '\t(paper "A3")', "\t(lib_symbols"]
    for name in LIBS:
        lines += lib_symbol(name)
    lines.append("\t)")
    lines.append('\t(text "Rovi M4 wiring -- ESP32-S3 + 2x TB6612FNG + 74HC123 dead-man + INA226 (bench draft)"'
                 f' (at 40.64 25.4 0) (effects (font (size 2.5 2.5)) (justify left)) (uuid "{uid("title")}"))')
    note = NOTES.replace('"', "'").replace("\n", "\\n")
    lines.append(f'\t(text "{note}" (at 40.64 208.28 0) (effects (font (size 1.4 1.4)) (justify left top))'
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
