#!/usr/bin/env python3
"""Generate the M4 build-guide illustrations (img/*.svg) and check the perfboard carrier.

The carrier layout below is the single source of truth for the build: part positions on the
2.54 mm perfboard grid and every wire on the solder side. `--check` traces those wires and
compares the result with the KiCad schematic's intended nets (../tools/gen_wiring.py PARTS), so
the board you solder is the circuit in the schematic.

  python3 build/gen_build_guide.py           # from experiments/rovi-m4-four-motor-safety/
  python3 build/gen_build_guide.py --check
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
IMG = os.path.join(HERE, "img")
sys.path.insert(0, os.path.join(HERE, "..", "tools"))
import gen_wiring  # noqa: E402  (PARTS: the intended nets)

# --- Carrier layout (top view: component side, ESP32 antenna up) -----------------------------
# Holes are (col, row), 1-based on a 35 x 27 hole perfboard (about 9 x 7 cm).
COLS, ROWS = 35, 27

TB_LEFT = ["VM", "VCC", "GND", "AO1", "AO2", "BO2", "BO1", "GND"]   # L1..L8, top to bottom
TB_RIGHT = ["PWMA", "AIN2", "AIN1", "STBY", "BIN1", "BIN2", "PWMB", "GND"]  # R1..R8

PINS = {}  # (ref, pin) -> (col, row)
for k in range(1, 23):
    PINS[("U1", f"L{k}")] = (20, k)   # ESP32 left header (L1 = 3V3 at the antenna end)
    PINS[("U1", f"R{k}")] = (30, k)   # ESP32 right header, 10 holes across (25.4 mm)
for ref, top in (("U2", 4), ("U3", 14)):  # driver 1 R1 beside ESP L4, driver 2 R1 beside L14
    for i in range(8):
        PINS[(ref, f"L{i + 1}")] = (10, top + i)  # power/motor header
        PINS[(ref, f"R{i + 1}")] = (16, top + i)  # control header, 6 holes across (15.24 mm)
PINS[("C1", "1")], PINS[("C1", "2")] = (6, 4), (6, 6)      # + to VM, - to GND
PINS[("C2", "1")], PINS[("C2", "2")] = (6, 14), (6, 16)
MOTOR_PADS = {"M1": (7, 8), "M2": (10, 9), "M3": (17, 18), "M4": (20, 19)}  # rows of lead 1, lead 2
for m, (r1, r2) in MOTOR_PADS.items():
    PINS[(m, "1")], PINS[(m, "2")] = (5, r1), (5, r2)  # via the motor's JST plug
PINS[("BT1", "1")], PINS[("BT1", "2")] = (4, 25), (2, 25)  # power-input pigtail: VM, GND

# Links that exist inside a board, not on the perfboard.
INTERNAL = [(("U1", "L1"), ("U1", "L2")), (("U1", "L22"), ("U1", "R1"))] + [
    ((ref, "L3"), (ref, p)) for ref in ("U2", "U3") for p in ("L8", "R8")]

RIBBON = ["#8B4513", "#E02020", "#FF8C00", "#E6C800", "#20A020", "#2060E0", "#8A2BE2"]
# Wires on the solder side, in soldering order: (from, to, colour, label). Ends are lap-soldered
# to the pin or lead on the solder side, so a pin can take more than one wire.
WIRES = []
for i, r in enumerate(range(4, 11)):
    WIRES.append((("U2", f"R{i + 1}"), ("U1", f"L{r}"), RIBBON[i], f"D1 {TB_RIGHT[i]}"))
for i, r in enumerate(range(14, 21)):
    WIRES.append((("U3", f"R{i + 1}"), ("U1", f"L{r}"), RIBBON[i], f"D2 {TB_RIGHT[i]}"))
WIRES += [
    (("U1", "L1"), ("U2", "L2"), "#F5F5F5", "3V3 to D1 VCC"),
    (("U1", "L2"), ("U3", "L2"), "#F5F5F5", "3V3 to D2 VCC"),
    (("C1", "1"), ("U2", "L1"), "#D00000", "VM: C1+ to D1 VM"),
    (("C2", "1"), ("U3", "L1"), "#D00000", "VM: C2+ to D2 VM"),
    (("C1", "1"), ("C2", "1"), "#D00000", "VM: C1+ to C2+"),
    (("BT1", "1"), ("C2", "1"), "#D00000", "VM in to C2+"),
    (("C1", "2"), ("U2", "L3"), "#202020", "GND: C1- to D1 GND"),
    (("C2", "2"), ("U3", "L3"), "#202020", "GND: C2- to D2 GND"),
    (("C1", "2"), ("C2", "2"), "#202020", "GND: C1- to C2-"),
    (("BT1", "2"), ("C2", "2"), "#202020", "GND in to C2-"),
    (("U1", "R1"), ("U2", "L3"), "#202020", "ESP32 GND (R1) to D1 GND"),
    (("U1", "L22"), ("U3", "L3"), "#202020", "ESP32 GND (L22) to D2 GND"),
]
for m, drv, pins in (("M1", "U2", ("L4", "L5")), ("M2", "U2", ("L7", "L6")),
                     ("M3", "U3", ("L4", "L5")), ("M4", "U3", ("L7", "L6"))):
    for k, p in zip(("1", "2"), pins):
        WIRES.append(((m, k), (drv, p), "#C8A000" if k == "1" else "#3A9A3A", f"{m} lead {k} to {drv} {p}"))


def check():
    """Traces the perfboard and compares every net with the schematic. Returns True if equal."""
    parent = {}

    def find(a):
        parent.setdefault(a, a)
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    for a, b in [(w[0], w[1]) for w in WIRES] + INTERNAL:
        parent[find(a)] = find(b)
    on_board = {ref for ref, _ in PINS}
    want = {}
    for ref, _, _, _, nets in gen_wiring.PARTS:
        if ref in on_board:
            for pin, net in nets.items():
                want.setdefault(net, set()).add((ref, pin))
    bad = 0
    for net, members in sorted(want.items()):
        roots = {find(m) for m in members}
        if len(roots) != 1:
            bad += 1
            print(f"net {net}: split into {len(roots)} groups on the perfboard")
    # No two schematic nets may be joined on the perfboard.
    owner = {}
    for net, members in want.items():
        for m in members:
            r = find(m)
            if owner.setdefault(r, net) != net:
                bad += 1
                print(f"nets {owner[r]} and {net} are shorted on the perfboard")
    # Every hole holds at most one part pin.
    holes = {}
    for k, v in PINS.items():
        if v in holes:
            bad += 1
            print(f"{k} and {holes[v]} share hole {v}")
        holes[v] = k
    print(f"{len(want)} nets on the carrier, {len(WIRES)} wires, {bad} problems")
    return bad == 0


# --- SVG helpers ------------------------------------------------------------------------------

def svg(w, h, body, title):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" '
            f'font-family="Helvetica, Arial, sans-serif">\n<title>{title}</title>\n'
            f'<rect width="{w}" height="{h}" fill="#ffffff"/>\n{body}\n</svg>\n')


def text(x, y, s, size=13, anchor="start", weight="normal", fill="#222", extra=""):
    s = s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    return (f'<text x="{x:.1f}" y="{y:.1f}" font-size="{size}" text-anchor="{anchor}" '
            f'font-weight="{weight}" fill="{fill}" {extra}>{s}</text>')


def write(name, content):
    with open(os.path.join(IMG, name), "w") as fh:
        fh.write(content)


# --- Perfboard views --------------------------------------------------------------------------

S = 22          # px per hole
OX, OY = 170, 140  # board origin


def hx(col, mirror=False):
    c = COLS + 1 - col if mirror else col
    return OX + (c - 0.5) * S


def hy(row):
    return OY + (row - 0.5) * S


def board(mirror):
    out = [f'<rect x="{OX}" y="{OY}" width="{COLS * S}" height="{ROWS * S}" rx="6" '
           f'fill="{"#C9A46B" if mirror else "#D9B77A"}" stroke="#8a6a35" stroke-width="2"/>']
    for c in range(1, COLS + 1):
        for r in range(1, ROWS + 1):
            out.append(f'<circle cx="{hx(c, mirror):.1f}" cy="{hy(r):.1f}" r="3.2" fill="#6b5330"/>')
            if mirror:
                out.append(f'<circle cx="{hx(c, mirror):.1f}" cy="{hy(r):.1f}" r="6" fill="none" '
                           f'stroke="#b07a3a" stroke-width="2"/>')
    for c in range(5, COLS + 1, 5):
        out.append(text(hx(c, mirror), OY - 8, str(c), 10, "middle", fill="#777"))
    for r in range(1, ROWS + 1, 2):
        out.append(text(OX - 10, hy(r) + 4, str(r), 10, "end", fill="#777"))
    return out


def pad(col, row, mirror, fill="#b8b8b8"):
    return f'<circle cx="{hx(col, mirror):.1f}" cy="{hy(row):.1f}" r="7" fill="{fill}" stroke="#666"/>'


def header_strip(col, r0, n, mirror, colour="#1a1a1a"):
    x = hx(col, mirror) - 9
    return (f'<rect x="{x:.1f}" y="{hy(r0) - 10:.1f}" width="18" height="{(n - 1) * S + 20}" rx="3" '
            f'fill="{colour}" opacity="0.92"/>')


def wire_path(a, b, mirror, bow):
    (c1, r1), (c2, r2) = PINS[a], PINS[b]
    x1, y1, x2, y2 = hx(c1, mirror), hy(r1), hx(c2, mirror), hy(r2)
    if r1 == r2 and abs(c2 - c1) <= 6:
        return f"M{x1:.1f},{y1:.1f} L{x2:.1f},{y2:.1f}"
    mx, my = (x1 + x2) / 2, (y1 + y2) / 2
    dx, dy = x2 - x1, y2 - y1
    ln = max((dx * dx + dy * dy) ** 0.5, 1)
    off = bow * S
    cx, cy = mx - dy / ln * off, my + dx / ln * off
    return f"M{x1:.1f},{y1:.1f} Q{cx:.1f},{cy:.1f} {x2:.1f},{y2:.1f}"


def top_view(name="carrier-top.svg", boxes=(), title=None):
    """The component side. `boxes` are hole rectangles (c0, r0, c1, r1, label) to highlight."""
    out = board(False)
    out.append(text(OX - 150, 30, title or "Carrier, TOP view (component side). Wires are on the other side: see the bottom view.", 15, weight="bold"))
    # ESP32 board outline (57 x 28 mm, antenna overhanging the top edge, USB-C at the bottom)
    x0, x1 = hx(20) - 0.55 * S, hx(30) + 0.55 * S
    out.append(f'<rect x="{x0:.1f}" y="{hy(1) - 0.75 * S - 3 * S:.1f}" width="{x1 - x0:.1f}" height="{3 * S:.1f}" '
               f'fill="#333" stroke="#000" opacity="0.85"/>')
    out.append(text((x0 + x1) / 2, hy(1) - 2.2 * S, "antenna: keep it off the board", 11, "middle", fill="#fff"))
    out.append(f'<rect x="{x0:.1f}" y="{hy(1) - 0.75 * S:.1f}" width="{x1 - x0:.1f}" height="{22.5 * S:.1f}" '
               f'fill="#1d1d1d" opacity="0.55" stroke="#000"/>')
    for c0 in (21.6, 25.6):
        out.append(f'<rect x="{hx(c0):.1f}" y="{hy(21.4):.1f}" width="{2.8 * S:.1f}" height="{1.6 * S:.1f}" rx="5" fill="#bbb" stroke="#555"/>')
    out.append(text((x0 + x1) / 2, hy(23.6), "2 × USB-C at this end: plug the ESP32 power bank into the COM port", 11, "middle"))
    out.append(text((x0 + x1) / 2, hy(24.3), "(the one you flash and log through)", 11, "middle"))
    out.append(text((x0 + x1) / 2, hy(11), "ESP32-S3 CORE", 16, "middle", "bold", "#fff"))
    out.append(text((x0 + x1) / 2, hy(12), "board", 14, "middle", fill="#fff"))
    out.append(header_strip(20, 1, 22, False))
    out.append(header_strip(30, 1, 22, False))
    for k in range(1, 23):
        lab = gen_wiring.ESP_LEFT[k - 1][1]
        out.append(text(hx(20) + 13, hy(k) + 4, f"L{k} {lab}", 10, fill="#ffef9e"))
    out.append(text(hx(30) - 13, hy(1) + 4, "R1 GND", 10, "end", fill="#ffef9e"))
    # Drivers
    for ref, top, dname in (("U2", 4, "DRIVER 1 (left wheels)"), ("U3", 14, "DRIVER 2 (right wheels)")):
        xa, xb = hx(10) - 0.9 * S, hx(16) + 0.9 * S
        out.append(f'<rect x="{xa:.1f}" y="{hy(top) - 0.8 * S:.1f}" width="{xb - xa:.1f}" height="{8.6 * S:.1f}" '
                   f'rx="4" fill="#d42a2a" opacity="0.75" stroke="#7a0000"/>')
        out.append(header_strip(10, top, 8, False))
        out.append(header_strip(16, top, 8, False))
        out.append(text((xa + xb) / 2, hy(top) - 0.95 * S, dname, 12, "middle", "bold"))
        for i in range(8):
            out.append(text(hx(10) + 13, hy(top + i) + 4, TB_LEFT[i], 10, fill="#fff"))
            out.append(text(hx(16) - 13, hy(top + i) + 4, TB_RIGHT[i], 10, "end", fill="#fff"))
    # Capacitors (seen from above: can with a stripe on the - side)
    for ref, (c, r) in (("C1", PINS[("C1", "1")]), ("C2", PINS[("C2", "1")])):
        cx, cy = hx(c), hy(r + 1)
        out.append(f'<circle cx="{cx - 0.2 * S:.1f}" cy="{cy:.1f}" r="{1.6 * S:.1f}" fill="#2b4fa8" stroke="#122a66" stroke-width="2"/>')
        out.append(f'<path d="M{cx - 1.5 * S:.1f},{cy + 0.6 * S:.1f} A{1.6 * S},{1.6 * S} 0 0 0 {cx + 1.1 * S:.1f},{cy + 0.9 * S:.1f}" '
                   f'stroke="#e8e8e8" stroke-width="7" fill="none"/>')
        out.append(text(cx - 0.2 * S, cy + 4, ref, 12, "middle", "bold", "#fff"))
        out.append(text(cx - 2.2 * S, cy - 0.5 * S, "+", 16, "middle", "bold"))
        out.append(text(cx - 2.2 * S, cy + 1.3 * S, "− stripe", 11, "middle"))
        out.append(pad(c, r, False, "#e0e0e0"))
        out.append(pad(c, r + 2, False, "#e0e0e0"))
    # Pigtails coming in from the left edge
    for m, (r1, r2) in MOTOR_PADS.items():
        for r, col in ((r1, "#C8A000"), (r2, "#3A9A3A")):
            out.append(f'<path d="M{OX - 45},{hy(r):.1f} L{hx(5):.1f},{hy(r):.1f}" stroke="{col}" stroke-width="4"/>')
            out.append(pad(5, r, False, "#e0e0e0"))
        out.append(text(OX - 50, (hy(r1) + hy(r2)) / 2 + 4, f"{m} plug", 12, "end", "bold"))
    for (c, r), col, lab in ((PINS[("BT1", "1")], "#D00000", "VM +5 V"), (PINS[("BT1", "2")], "#202020", "GND")):
        out.append(f'<path d="M{hx(c):.1f},{OY + ROWS * S + 40} L{hx(c):.1f},{hy(r):.1f}" stroke="{col}" stroke-width="4"/>')
        out.append(pad(c, r, False, "#e0e0e0"))
    out.append(text(hx(3), OY + ROWS * S + 58, "power-in plug (3-pin): from the motor power bank", 12, "middle", "bold"))
    out.append(text(OX - 50, hy(3), "motor plugs (2-pin)", 12, "end", "bold"))
    out.append(text(OX + COLS * S, OY + ROWS * S + 25, "zip-tie the pigtails through free holes for strain relief", 11, "end", fill="#555"))
    for c0, r0, c1, r1, lab in boxes:
        x, y = OX + (c0 - 1) * S - 4, OY + (r0 - 1) * S - 4
        w, h = (c1 - c0 + 1) * S + 8, (r1 - r0 + 1) * S + 8
        out.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="8" fill="#ffe000" fill-opacity="0.22" '
                   f'stroke="#ff8c00" stroke-width="4" stroke-dasharray="10,5"/>')
        if lab:
            out.append(f'<rect x="{x}" y="{y - 24}" width="{len(lab) * 8 + 16}" height="22" rx="5" fill="#ff8c00"/>')
            out.append(text(x + 8, y - 8, lab, 13, weight="bold", fill="#fff"))
    write(name, svg(OX + COLS * S + 60, OY + ROWS * S + 80, "\n".join(out), "Carrier top view"))


def bottom_view(name="carrier-bottom.svg", only=None, title=None):
    """The solder side. With `only` (wire numbers), the other wires are dimmed and unnumbered."""
    out = board(True)
    out.append(text(OX - 150, 30, title or "Carrier, BOTTOM view (solder side, flipped left-right). Solder the wires in the numbered order.", 15, weight="bold"))
    out.append(text(OX - 150, 52, "Flip the board over its long side: the ESP32 headers are now on the LEFT.", 13, fill="#555"))
    used = {}
    for (ref, pin), (c, r) in PINS.items():
        used[(c, r)] = (ref, pin)
    for (c, r) in used:
        out.append(pad(c, r, True, "#c0c0c0"))
    bows = {}
    for n, (a, b, colour, label) in enumerate(WIRES, 1):
        key = (PINS[a][0], PINS[b][0])
        bows[key] = bows.get(key, 0) + 1
        bow = 0.12 * ((bows[key] % 3) - 1)
        if label.startswith(("ESP32 GND", "3V3")):
            bow = 0.25
        if label == "VM: C1+ to C2+":
            bow = -1.6
        if label == "GND: C1- to C2-":
            bow = 1.2
        d = wire_path(a, b, True, bow)
        dim = only is not None and n not in only
        op = ' opacity="0.18"' if dim else ""
        out.append(f'<path d="{d}" stroke="#000" stroke-width="6" fill="none" stroke-linecap="round" opacity="{0.08 if dim else 0.5}"/>')
        out.append(f'<path d="{d}" stroke="{colour}" stroke-width="{4 if dim else 5}" fill="none" stroke-linecap="round"{op}/>')
    for n, (a, b, colour, label) in enumerate(WIRES, 1):
        if only is not None and n not in only:
            continue
        (c1, r1), (c2, r2) = PINS[a], PINS[b]
        mx, my = (hx(c1, True) + hx(c2, True)) / 2, (hy(r1) + hy(r2)) / 2
        out.append(f'<circle cx="{mx:.1f}" cy="{my:.1f}" r="8" fill="#fff" stroke="#333"/>')
        out.append(text(mx, my + 3.5, str(n), 9, "middle", "bold"))
    wired = {w[0] for w in WIRES} | {w[1] for w in WIRES}
    for (c, r), (ref, pin) in used.items():
        if (ref, pin) not in wired:
            continue
        if only is not None and not any((ref, pin) in (WIRES[n - 1][0], WIRES[n - 1][1]) for n in only):
            continue
        if ref == "U1":
            if pin.startswith("R") or int(pin[1:]) in range(4, 11) or int(pin[1:]) in range(14, 21):
                lab = pin if pin != "R1" else "R1 GND"
            else:
                lab = {"L1": "L1 3V3", "L2": "L2 3V3", "L22": "L22 GND"}.get(pin, pin)
            if pin.startswith("R") and pin != "R1":
                continue
        elif ref in ("U2", "U3"):
            i = int(pin[1:]) - 1
            lab = ("D1 " if ref == "U2" else "D2 ") + (TB_LEFT[i] if pin[0] == "L" else TB_RIGHT[i])
        elif ref in ("C1", "C2"):
            lab = ref + ("+" if pin == "1" else "−")
        elif ref == "BT1":
            lab = "VM in" if pin == "1" else "GND in"
        else:
            lab = f"{ref}.{pin}"
        out.append(text(hx(c, True) + 9, hy(r) - 7, lab, 8, weight="bold", fill="#1a1a1a"))
    for xcol, lab in ((20, "ESP32 left header"), (30, "ESP32 right header"), (16, "driver control"), (10, "driver power"),
                      (5.5, "caps + pigtails")):
        out.append(text(hx(xcol, True), OY - 24, lab, 11, "middle", "bold"))
    write(name, svg(OX + COLS * S + 60, OY + ROWS * S + 30, "\n".join(out), "Carrier bottom view"))


# --- Parts dictionary ---------------------------------------------------------------------------

def card(x, y, w, h, title, lines, art):
    out = [f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="10" fill="#fafafa" stroke="#bbb"/>',
           text(x + 12, y + 24, title, 15, weight="bold")]
    out += art
    for i, ln in enumerate(lines):
        out.append(text(x + 12, y + h - 12 - (len(lines) - 1 - i) * 16, ln, 12, fill="#333"))
    return out


def parts():
    W, H, P = 300, 250, 16
    items, count = [], [0]

    def add(title, lines, art_fn):
        i = count[0]
        count[0] += 1
        x, y = P + (i % 4) * (W + P), 60 + (i // 4) * (H + P)
        items.extend(card(x, y, W, H, title, lines, art_fn(x, y)))

    add("ESP32-S3 CORE board ×1", ["The brain. Plugs into the carrier.", "Have it."],
        lambda x, y: [f'<rect x="{x+40}" y="{y+50}" width="220" height="100" rx="6" fill="#1d1d1d"/>',
                      f'<rect x="{x+20}" y="{y+70}" width="24" height="60" fill="#333"/>',
                      f'<rect x="{x+90}" y="{y+68}" width="60" height="62" fill="#aaa"/>',
                      f'<rect x="{x+225}" y="{y+70}" width="30" height="22" rx="5" fill="#bbb"/>',
                      f'<rect x="{x+225}" y="{y+108}" width="30" height="22" rx="5" fill="#bbb"/>']
                     + [f'<circle cx="{x+50+i*9.5}" cy="{y+56}" r="2.5" fill="#d4b04a"/>' for i in range(22)]
                     + [f'<circle cx="{x+50+i*9.5}" cy="{y+144}" r="2.5" fill="#d4b04a"/>' for i in range(22)])
    add("TB6612FNG driver ×2", ["Drives two motors each.", "Have them (M1 used one)."],
        lambda x, y: [f'<rect x="{x+95}" y="{y+45}" width="110" height="110" rx="5" fill="#d42a2a"/>',
                      f'<rect x="{x+130}" y="{y+80}" width="40" height="40" fill="#222"/>']
                     + [f'<circle cx="{x+108}" cy="{y+56+i*13}" r="3" fill="#d4b04a"/>' for i in range(8)]
                     + [f'<circle cx="{x+192}" cy="{y+56+i*13}" r="3" fill="#d4b04a"/>' for i in range(8)])
    add("TT gear motor ×4", ["2 wires, no encoder.", "On the chassis."],
        lambda x, y: [f'<rect x="{x+60}" y="{y+70}" width="120" height="60" rx="8" fill="#f0d000"/>',
                      f'<rect x="{x+180}" y="{y+80}" width="70" height="40" rx="18" fill="#b0b0b0"/>',
                      f'<rect x="{x+250}" y="{y+88}" width="10" height="8" fill="#c08040"/>',
                      f'<rect x="{x+250}" y="{y+104}" width="10" height="8" fill="#c08040"/>',
                      f'<rect x="{x+20}" y="{y+95}" width="40" height="10" fill="#fff" stroke="#999"/>'])
    add("Power bank 5 V USB ×2", ["One for the motors (VM),", "one for the ESP32 (USB-C)."],
        lambda x, y: [f'<rect x="{x+70}" y="{y+45}" width="160" height="110" rx="16" fill="#f4f4f4" stroke="#999" stroke-width="2"/>',
                      f'<rect x="{x+222}" y="{y+85}" width="16" height="30" fill="#666"/>',
                      text(x + 150, y + 105, "5 V", 22, "middle", "bold", "#888")])
    add("JST-SM 2-pin pair ×4", ["Motor plugs (your AliExpress set).", "Pin side on motor, socket side on carrier."],
        lambda x, y: jst_art(x + 30, y + 60, 2))
    add("JST-SM 3-pin pair ×1", ["Power-in plug. 3-pin so it can't", "mate with a motor plug. Middle empty."],
        lambda x, y: jst_art(x + 30, y + 55, 3))
    add("Crimp terminals", ["Come with the JST set: one per wire.", "Pins (male) and sockets (female)."],
        lambda x, y: terminal_art(x + 40, y + 90) + terminal_art(x + 40, y + 140, female=True))
    add("Electrolytic 220-470 µF ×2", ["C1, C2 at each driver's VM. ≥ 10 V.", "Stripe = − leg (shorter leg)."],
        lambda x, y: [f'<rect x="{x+120}" y="{y+45}" width="60" height="90" rx="10" fill="#2b4fa8"/>',
                      f'<rect x="{x+165}" y="{y+45}" width="15" height="90" fill="#e8e8e8"/>',
                      text(x + 172, y + 95, "−", 18, "middle", "bold", "#2b4fa8"),
                      f'<line x1="{x+135}" y1="{y+135}" x2="{x+135}" y2="{y+175}" stroke="#999" stroke-width="3"/>',
                      f'<line x1="{x+165}" y1="{y+135}" x2="{x+165}" y2="{y+165}" stroke="#999" stroke-width="3"/>',
                      text(x + 128, y + 188, "+ long", 11, "end"), text(x + 172, y + 178, "− short", 11)])
    add("Ceramic 100 nF ×4", ["Marked 104. Across each motor's tabs.", "No polarity."],
        lambda x, y: [f'<ellipse cx="{x+150}" cy="{y+95}" rx="26" ry="20" fill="#e08a1a"/>',
                      text(x + 150, y + 100, "104", 13, "middle", "bold"),
                      f'<line x1="{x+140}" y1="{y+113}" x2="{x+135}" y2="{y+170}" stroke="#999" stroke-width="3"/>',
                      f'<line x1="{x+160}" y1="{y+113}" x2="{x+165}" y2="{y+170}" stroke="#999" stroke-width="3"/>'])
    add("Perfboard ≥ 9 × 7 cm ×1", ["Plain holes, 2.54 mm pitch.", "35 × 27 holes or more."],
        lambda x, y: [f'<rect x="{x+50}" y="{y+45}" width="200" height="130" rx="6" fill="#D9B77A"/>']
                     + [f'<circle cx="{x+60+i*12}" cy="{y+55+j*12}" r="2" fill="#6b5330"/>' for i in range(16) for j in range(10)])
    add("Female header strips", ["2 × 22-pin (ESP32), 4 × 8-pin (drivers).", "Cut longer strips to length."],
        lambda x, y: [f'<rect x="{x+40}" y="{y+70}" width="220" height="22" rx="3" fill="#1a1a1a"/>',
                      f'<rect x="{x+40}" y="{y+120}" width="90" height="22" rx="3" fill="#1a1a1a"/>']
                     + [f'<rect x="{x+46+i*10}" y="{y+77}" width="5" height="8" fill="#888"/>' for i in range(21)]
                     + [f'<rect x="{x+46+i*10}" y="{y+127}" width="5" height="8" fill="#888"/>' for i in range(8)])
    add("Wire", ["Solid 22 AWG for the carrier (several colours).", "Stranded 22-24 AWG red/black for leads."],
        lambda x, y: [f'<path d="M{x+30},{y+70+i*18} C{x+120},{y+40+i*18} {x+180},{y+100+i*18} {x+270},{y+70+i*18}" '
                      f'stroke="{c}" stroke-width="5" fill="none"/>' for i, c in enumerate(RIBBON[:6])])
    add("USB cable to cut ×1", ["USB-A plug end for the motor power bank.", "Red = +5 V, black = GND (check!)."],
        lambda x, y: [f'<rect x="{x+40}" y="{y+85}" width="50" height="30" rx="3" fill="#bbb" stroke="#666"/>',
                      f'<path d="M{x+90},{y+100} L{x+180},{y+100}" stroke="#222" stroke-width="9"/>',
                      f'<path d="M{x+180},{y+100} L{x+250},{y+80}" stroke="#D00000" stroke-width="4"/>',
                      f'<path d="M{x+180},{y+100} L{x+250},{y+120}" stroke="#202020" stroke-width="4"/>',
                      f'<path d="M{x+180},{y+100} L{x+240},{y+100}" stroke="#20A020" stroke-width="3" opacity="0.5"/>',
                      f'<path d="M{x+180},{y+100} L{x+235},{y+110}" stroke="#ffffff" stroke-width="3" opacity="0.5"/>',
                      text(x + 252, y + 104, "data: cut short", 10)])
    add("Heat-shrink, zip ties", ["Heat-shrink 2-3 mm over joints.", "Zip ties for strain relief."],
        lambda x, y: [f'<rect x="{x+60}" y="{y+70}" width="150" height="14" rx="7" fill="#222"/>',
                      f'<rect x="{x+60}" y="{y+100}" width="110" height="10" rx="5" fill="#c00"/>',
                      f'<path d="M{x+80},{y+150} L{x+240},{y+150}" stroke="#eee" stroke-width="8"/>',
                      f'<path d="M{x+80},{y+150} L{x+240},{y+150}" stroke="#bbb" stroke-width="2" stroke-dasharray="3,3"/>'])
    add("USB-C cable ×1", ["ESP32 power bank to the ESP32's", "COM USB-C port."],
        lambda x, y: [f'<rect x="{x+40}" y="{y+88}" width="40" height="24" rx="3" fill="#bbb"/>',
                      f'<path d="M{x+80},{y+100} C{x+150},{y+40} {x+170},{y+160} {x+230},{y+100}" stroke="#222" stroke-width="8" fill="none"/>',
                      f'<rect x="{x+230}" y="{y+92}" width="34" height="16" rx="6" fill="#bbb"/>'])
    add("Tools", ["Soldering iron, solder, crimper (SN-28B type),", "wire stripper, flush cutter, multimeter."],
        lambda x, y: [f'<path d="M{x+40},{y+150} L{x+150},{y+60}" stroke="#444" stroke-width="12" stroke-linecap="round"/>',
                      f'<path d="M{x+150},{y+60} L{x+170},{y+44}" stroke="#bbb" stroke-width="4"/>',
                      f'<path d="M{x+180},{y+160} L{x+230},{y+70} M{x+260},{y+160} L{x+230},{y+70}" stroke="#c00" stroke-width="10" stroke-linecap="round"/>'])
    rows = (len(items) and 4)
    write("parts.svg", svg(4 * (W + P) + P, 60 + 4 * (H + P), "\n".join(
        [text(P, 38, "Parts dictionary: everything on the bench for M4 step 2", 20, weight="bold")] + items), "Parts dictionary"))


def jst_art(x, y, n):
    out = []
    w = 26 * n + 10
    out.append(f'<rect x="{x}" y="{y}" width="{w}" height="40" rx="4" fill="#f2f2f2" stroke="#888"/>')
    for i in range(n):
        out.append(f'<rect x="{x+8+i*26}" y="{y+10}" width="18" height="20" fill="#ccc"/>')
        out.append(f'<rect x="{x+14+i*26}" y="{y+40}" width="6" height="18" fill="#d4b04a"/>')
    out.append(f'<rect x="{x+w/2-8}" y="{y-8}" width="16" height="8" fill="#ddd" stroke="#888"/>')
    out.append(text(x + w / 2, y + 112, "pin side", 11, "middle"))
    xs = x + w + 40
    out.append(f'<rect x="{xs}" y="{y}" width="{w}" height="40" rx="4" fill="#f2f2f2" stroke="#888"/>')
    for i in range(n):
        out.append(f'<rect x="{xs+8+i*26}" y="{y+10}" width="18" height="20" fill="#555"/>')
    out.append(f'<rect x="{xs+w/2-10}" y="{y-10}" width="20" height="10" fill="#f2f2f2" stroke="#888"/>')
    out.append(text(xs + w / 2, y + 112, "socket side", 11, "middle"))
    for i in range(n):
        if n == 3 and i == 1:
            continue
        for xx, yy in ((x + 17 + i * 26, y + 58), (xs + 17 + i * 26, y + 40)):
            out.append(f'<path d="M{xx},{yy} L{xx},{yy+40}" stroke="{"#D00000" if i == 0 else "#202020"}" stroke-width="3"/>')
    return out


def terminal_art(x, y, female=False):
    out = [f'<rect x="{x}" y="{y-6}" width="{"50" if female else "40"}" height="12" rx="2" fill="#d4b04a" stroke="#9a7a20"/>']
    if not female:
        out.append(f'<rect x="{x-30}" y="{y-2}" width="30" height="4" fill="#d4b04a"/>')
    out += [f'<path d="M{x+55},{y-8} l12,-8 M{x+55},{y+8} l12,8" stroke="#9a7a20" stroke-width="3"/>',
            f'<path d="M{x+80},{y-10} l14,-10 M{x+80},{y+10} l14,10" stroke="#9a7a20" stroke-width="3"/>',
            f'<rect x="{x+50}" y="{y-4}" width="50" height="8" fill="#d4b04a"/>',
            text(x + 58, y - 22, "conductor", 9, "end"), text(x + 84, y - 22, "insulation", 9),
            text(x + 200, y + 4, "socket" if female else "pin", 11, "end")]
    return out


# --- Crimping, harness, power cable, system ---------------------------------------------------

def crimp():
    W, H = 280, 250
    steps = [
        ("1. Strip 2-2.5 mm", ["Bare copper fills the conductor wings only.", ""], lambda x, y: [
            f'<path d="M{x+30},{y+120} L{x+170},{y+120}" stroke="#D00000" stroke-width="12"/>',
            f'<path d="M{x+170},{y+120} L{x+215},{y+120}" stroke="#c87533" stroke-width="5"/>',
            f'<line x1="{x+170}" y1="{y+95}" x2="{x+170}" y2="{y+145}" stroke="#666" stroke-dasharray="4,3"/>',
            f'<line x1="{x+215}" y1="{y+95}" x2="{x+215}" y2="{y+145}" stroke="#666" stroke-dasharray="4,3"/>',
            text(x + 192, y + 90, "2-2.5 mm", 12, "middle")]),
        ("2. Terminal in the die", ["Wings up, in the smallest slot that closes", "both wing pairs (match your wire gauge)."], lambda x, y: [
            f'<rect x="{x+40}" y="{y+70}" width="200" height="90" rx="8" fill="#555"/>',
            f'<rect x="{x+60}" y="{y+100}" width="40" height="24" fill="#999"/>',
            f'<rect x="{x+110}" y="{y+100}" width="40" height="24" fill="#999"/>',
            f'<rect x="{x+160}" y="{y+100}" width="40" height="24" fill="#999"/>',
            f'<rect x="{x+112}" y="{y+106}" width="36" height="10" fill="#d4b04a"/>',
            text(x + 130, y + 150, "pick the slot that closes the wings", 10, "middle", fill="#fff")]),
        ("3. Wire in, squeeze fully", ["Ratchet until it releases. Copper under the", "conductor wings, insulation under the others."], lambda x, y: terminal_art(x + 60, y + 120) + [
            f'<path d="M{x+170},{y+120} L{x+260},{y+120}" stroke="#D00000" stroke-width="10"/>',
            ]),
        ("4. Check the crimp", ["Wings rolled tight, copper visible at both", "ends; a firm tug must not pull it out."], lambda x, y: [
            f'<rect x="{x+40}" y="{y+80}" width="60" height="14" rx="3" fill="#d4b04a"/>',
            f'<rect x="{x+100}" y="{y+82}" width="30" height="10" fill="#c87533"/>',
            f'<rect x="{x+130}" y="{y+78}" width="30" height="18" rx="4" fill="#b88a2a"/>',
            f'<path d="M{x+160},{y+87} L{x+250},{y+87}" stroke="#D00000" stroke-width="10"/>',
            text(x + 150, y + 125, "✔ good", 14, "middle", "bold", "#20A020"),
            f'<rect x="{x+40}" y="{y+150}" width="60" height="14" rx="3" fill="#d4b04a"/>',
            f'<path d="M{x+100},{y+157} L{x+250},{y+157}" stroke="#D00000" stroke-width="10"/>',
            text(x + 150, y + 190, "✘ insulation in the conductor wings", 12, "middle", "bold", "#c00"),
            ]),
        ("5. Into the housing", ["Barb toward the latch window; push until it", "clicks. It must not pull back out."], lambda x, y: jst_art(x + 50, y + 70, 2)[:5] + [
            f'<path d="M{x+90},{y+200} L{x+90},{y+140}" stroke="#D00000" stroke-width="4" marker-end="url(#arr)"/>',
            ]),
        ("6. Mate and test", ["Click the halves together. Check polarity", "with a multimeter before plugging power."], lambda x, y: jst_art(x + 30, y + 60, 2)),
    ]
    out = ['<defs><marker id="arr" markerWidth="10" markerHeight="10" refX="5" refY="5" orient="auto">'
           '<path d="M0,0 L10,5 L0,10 z" fill="#D00000"/></marker></defs>',
           text(16, 38, "Crimping a JST-SM terminal (open-barrel, two-stage crimp)", 20, weight="bold")]
    for i, (title, lines, fn) in enumerate(steps):
        x, y = 16 + (i % 3) * (W + 16), 60 + (i // 3) * (H + 16)
        out += card(x, y, W, H, title, lines, fn(x, y))
    write("crimp.svg", svg(3 * (W + 16) + 16, 60 + 2 * (H + 16), "\n".join(out), "Crimping"))


def harness():
    out = [text(16, 36, "Motor harness (×4) and carrier pigtail", 20, weight="bold")]
    y = 150
    out += [f'<rect x="40" y="{y-45}" width="150" height="90" rx="10" fill="#f0d000"/>',
            text(115, y + 5, "TT motor", 15, "middle", "bold"),
            f'<rect x="190" y="{y-30}" width="40" height="60" rx="6" fill="#b0b0b0"/>',
            f'<rect x="230" y="{y-22}" width="14" height="10" fill="#c08040"/>',
            f'<rect x="230" y="{y+12}" width="14" height="10" fill="#c08040"/>',
            f'<ellipse cx="262" cy="{y}" rx="11" ry="9" fill="#e08a1a"/>',
            f'<path d="M244,{y-17} L258,{y-8} M244,{y+17} L258,{y+8}" stroke="#999" stroke-width="2"/>',
            text(262, y - 30, "100 nF across the tabs", 12, "middle"),
            f'<path d="M244,{y-17} C330,{y-17} 340,{y+17} 420,{y-6} S520,{y+12} 600,{y-7}" stroke="#C8A000" stroke-width="5" fill="none"/>',
            f'<path d="M244,{y+17} C330,{y+17} 340,{y-17} 420,{y+6} S520,{y-12} 600,{y+7}" stroke="#3A9A3A" stroke-width="5" fill="none"/>',
            text(420, y + 45, "twist the pair, ~15-25 cm", 12, "middle"),
            f'<rect x="250" y="{y-30}" width="30" height="60" fill="#222" opacity="0.25"/>',
            text(265, y + 50, "heat-shrink", 11, "middle")]
    # Pin-side housing on the motor leads: wires enter at the back (left), pins face right.
    out += [f'<rect x="600" y="{y-24}" width="60" height="48" rx="5" fill="#f2f2f2" stroke="#888"/>',
            f'<rect x="616" y="{y-30}" width="22" height="6" fill="#ddd" stroke="#888"/>',
            f'<rect x="660" y="{y-10}" width="22" height="5" fill="#d4b04a"/>',
            f'<rect x="660" y="{y+5}" width="22" height="5" fill="#d4b04a"/>',
            text(630, y + 50, "PIN side", 13, "middle", "bold"), text(630, y + 66, "(on the motor)", 12, "middle"),
            # Socket-side housing on the carrier pigtail: sockets face left, wires leave at the back.
            f'<rect x="700" y="{y-24}" width="60" height="48" rx="5" fill="#f2f2f2" stroke="#888"/>',
            f'<rect x="700" y="{y-12}" width="14" height="9" fill="#555"/>',
            f'<rect x="700" y="{y+3}" width="14" height="9" fill="#555"/>',
            f'<rect x="716" y="{y-32}" width="28" height="8" fill="#f2f2f2" stroke="#888"/>',
            text(730, y + 50, "SOCKET side", 13, "middle", "bold"), text(730, y + 66, "(carrier pigtail)", 12, "middle"),
            f'<path d="M682,{y-36} L700,{y-36}" stroke="#888" stroke-width="2"/>', text(691, y - 42, "click", 10, "middle"),
            f'<path d="M760,{y-6} L900,{y-6}" stroke="#C8A000" stroke-width="5"/>',
            f'<path d="M760,{y+6} L900,{y+6}" stroke="#3A9A3A" stroke-width="5"/>',
            f'<rect x="900" y="{y-40}" width="70" height="80" fill="#D9B77A"/>',
            text(935, y + 60, "carrier", 12, "middle"),
            text(16, y + 110, "• Socket (recessed metal) on the side that carries power: carrier → motor, power bank → carrier.", 13),
            text(16, y + 130, "• Motor polarity does not matter for safety: a reversed motor just spins backwards; the firmware inverts it.", 13),
            text(16, y + 150, "• Pigtail on the carrier: ~8 cm, soldered into holes (col 5, see the carrier views), zip-tied for strain relief.", 13)]
    write("harness.svg", svg(1000, 320, "\n".join(out), "Motor harness"))


def power():
    out = [text(16, 36, "System: two power banks, one carrier", 20, weight="bold"),
           f'<rect x="30" y="80" width="190" height="110" rx="16" fill="#f4f4f4" stroke="#999" stroke-width="2"/>',
           text(125, 125, "MOTOR", 16, "middle", "bold"), text(125, 148, "power bank", 14, "middle"),
           f'<rect x="30" y="260" width="190" height="110" rx="16" fill="#f4f4f4" stroke="#999" stroke-width="2"/>',
           text(125, 305, "ESP32", 16, "middle", "bold"), text(125, 328, "power bank", 14, "middle"),
           f'<path d="M220,135 L340,135" stroke="#222" stroke-width="8"/>',
           text(280, 120, "USB-A cable, cut", 12, "middle"),
           f'<path d="M340,135 L420,128" stroke="#D00000" stroke-width="4"/><path d="M340,135 L420,142" stroke="#202020" stroke-width="4"/>',
           f'<rect x="420" y="115" width="44" height="40" rx="4" fill="#f2f2f2" stroke="#888"/>',
           text(442, 175, "3-pin JST (socket)", 11, "middle"),
           f'<path d="M464,128 L560,128" stroke="#D00000" stroke-width="4"/><path d="M464,142 L560,142" stroke="#202020" stroke-width="4"/>',
           f'<path d="M220,315 L560,315" stroke="#222" stroke-width="8"/>',
           text(390, 300, "USB-C cable → ESP32 COM port", 12, "middle"),
           f'<rect x="560" y="80" width="300" height="300" rx="8" fill="#D9B77A" stroke="#8a6a35" stroke-width="2"/>',
           text(710, 105, "CARRIER (perfboard)", 15, "middle", "bold"),
           f'<rect x="580" y="120" width="90" height="100" fill="#d42a2a" opacity="0.8"/>', text(625, 175, "D1", 16, "middle", "bold", "#fff"),
           f'<rect x="580" y="240" width="90" height="100" fill="#d42a2a" opacity="0.8"/>', text(625, 295, "D2", 16, "middle", "bold", "#fff"),
           f'<rect x="720" y="115" width="120" height="240" fill="#1d1d1d" opacity="0.8"/>', text(780, 240, "ESP32", 16, "middle", "bold", "#fff"),
           f'<path d="M670,140 L720,140 M670,155 L720,155 M670,170 L720,170 M670,185 L720,185 M670,200 L720,200" stroke="#ffcc00" stroke-width="2"/>',
           f'<path d="M670,260 L720,260 M670,275 L720,275 M670,290 L720,290 M670,305 L720,305 M670,320 L720,320" stroke="#ffcc00" stroke-width="2"/>',
           text(695, 365, "14 signal wires", 11, "middle")]
    for i, (lab, yy) in enumerate((("front-left", 130), ("rear-left", 190), ("front-right", 250), ("rear-right", 310))):
        out += [f'<path d="M560,{yy} L500,{yy+60*0}" stroke="#C8A000" stroke-width="0"/>']
    for i, (lab, yy) in enumerate((("FL", 420), ("RL", 470), ("FR", 520), ("RR", 570))):
        out += [f'<rect x="{570+i*75}" y="430" width="60" height="40" rx="6" fill="#f0d000"/>',
                text(600 + i * 75, 455, lab, 13, "middle", "bold"),
                f'<path d="M{600+i*75},430 L{600+i*75},380" stroke="#3A9A3A" stroke-width="4"/>']
    out += [text(710, 495, "4 motors via 2-pin JST plugs", 12, "middle"),
            text(16, 420, "Kill switch: unplug the motor", 13, "start", "bold"),
            text(16, 438, "power bank. The ESP32 stays up.", 13),
            text(16, 470, "Grounds join on the carrier.", 13)]
    write("system.svg", svg(900, 520, "\n".join(out), "System overview"))


COLOUR_NAMES = dict(zip(RIBBON, ["brown", "red", "orange", "yellow", "green", "blue", "violet"]))
COLOUR_NAMES.update({"#F5F5F5": "white", "#D00000": "red", "#202020": "black", "#C8A000": "yellow", "#3A9A3A": "green"})


def pin_name(ref, pin):
    if ref == "U1":
        return f"ESP32 {pin} (GPIO {gen_wiring.ESP_LEFT[int(pin[1:]) - 1][1]})" if pin[0] == "L" and pin not in ("L1", "L2", "L22") \
            else {"L1": "ESP32 L1 (3V3)", "L2": "ESP32 L2 (3V3)", "L22": "ESP32 L22 (GND)", "R1": "ESP32 R1 (GND)"}[pin]
    if ref in ("U2", "U3"):
        i = int(pin[1:]) - 1
        return ("Driver 1 " if ref == "U2" else "Driver 2 ") + f"{pin} ({TB_LEFT[i] if pin[0] == 'L' else TB_RIGHT[i]})"
    if ref in ("C1", "C2"):
        return f"{ref} {'+' if pin == '1' else '−'} leg"
    if ref == "BT1":
        return "power-in pigtail " + ("VM (red)" if pin == "1" else "GND (black)")
    return f"{ref} pigtail lead {pin}"


def wire_list():
    rows = ["# Carrier wire list", "",
            "Generated by `gen_build_guide.py` from the same data as the drawings; `--check` proves it matches the schematic.",
            "Holes are (column, row) in the **top view** numbering; the bottom view shows the same numbers mirrored.", "",
            "| # | Colour | From | Hole | To | Hole |", "| --- | --- | --- | --- | --- | --- |"]
    for n, (a, b, colour, _) in enumerate(WIRES, 1):
        rows.append(f"| {n} | {COLOUR_NAMES[colour]} | {pin_name(*a)} | {PINS[a]} | {pin_name(*b)} | {PINS[b]} |")
    with open(os.path.join(HERE, "wire-list.md"), "w") as fh:
        fh.write("\n".join(rows) + "\n")


def power_cable():
    out = [text(16, 36, "Motor power cable and the carrier's power-in pigtail", 20, weight="bold"),
           f'<rect x="30" y="90" width="80" height="44" rx="4" fill="#bbb" stroke="#666"/>',
           text(70, 117, "USB-A", 13, "middle", "bold"),
           f'<path d="M110,112 L300,112" stroke="#222" stroke-width="10"/>',
           text(205, 98, "~30 cm, outer jacket stripped 4 cm", 12, "middle"),
           f'<path d="M300,112 L420,92" stroke="#D00000" stroke-width="5"/>',
           f'<path d="M300,112 L420,132" stroke="#202020" stroke-width="5"/>',
           f'<path d="M300,112 L350,150 M300,112 L345,160" stroke="#20A020" stroke-width="3"/>',
           f'<path d="M345,160 L352,152" stroke="#20A020" stroke-width="3"/>',
           f'<rect x="338" y="145" width="18" height="22" rx="3" fill="#222" opacity="0.6"/>',
           text(360, 182, "data wires: cut short, heat-shrink each", 11, "middle"),
           f'<rect x="420" y="76" width="64" height="74" rx="5" fill="#f2f2f2" stroke="#888"/>',
           f'<rect x="420" y="84" width="14" height="14" fill="#555"/>', text(442, 96, "1", 12),
           f'<rect x="420" y="106" width="14" height="14" fill="#ddd"/>', text(442, 118, "2 empty", 11),
           f'<rect x="420" y="128" width="14" height="14" fill="#555"/>', text(442, 140, "3", 12),
           f'<circle cx="478" cy="84" r="6" fill="#D00000"/>',
           text(452, 172, "3-pin SOCKET side", 13, "middle", "bold"),
           text(452, 188, "red = 1, black = 3", 12, "middle"),
           f'<path d="M500,113 L540,113" stroke="#888" stroke-width="2" stroke-dasharray="4,3"/>', text(520, 104, "click", 10, "middle"),
           f'<rect x="540" y="76" width="64" height="74" rx="5" fill="#f2f2f2" stroke="#888"/>',
           f'<rect x="604" y="86" width="16" height="8" fill="#d4b04a"/>', f'<rect x="604" y="130" width="16" height="8" fill="#d4b04a"/>',
           f'<circle cx="548" cy="84" r="6" fill="#D00000"/>',
           text(572, 172, "3-pin PIN side", 13, "middle", "bold"), text(572, 188, "(carrier pigtail)", 12, "middle"),
           f'<path d="M620,90 L760,90" stroke="#D00000" stroke-width="5"/>', f'<path d="M620,134 L760,134" stroke="#202020" stroke-width="5"/>',
           f'<rect x="760" y="60" width="90" height="100" fill="#D9B77A"/>', text(805, 178, "carrier", 12, "middle"),
           text(805, 105, "VM (4,25)", 11, "middle"), text(805, 125, "GND (2,25)", 11, "middle"),
           text(16, 230, "1. Plug the USB-A end into the motor power bank and MEASURE which wire carries +5 V (usually red). Unplug.", 13),
           text(16, 250, "2. Pin 1 = +5 V (red), pin 3 = GND (black), pin 2 empty, on both halves. Mark pin 1 with a red dot.", 13),
           text(16, 270, "3. The 3-pin plug cannot mate with a 2-pin motor plug, and the keyed housing only fits one way round.", 13)]
    write("power-cable.svg", svg(880, 290, "\n".join(out), "Motor power cable"))


# Per-step views: (file, title, highlight boxes) and (file, title, wire numbers).
TOP_STEPS = [
    ("step-headers.svg", "Step: solder the female header sockets (boards plugged in while soldering)",
     [(20, 1, 20, 22, "ESP32 left"), (30, 1, 30, 22, "ESP32 right"), (10, 4, 10, 11, "D1"), (16, 4, 16, 11, ""),
      (10, 14, 10, 21, "D2"), (16, 14, 16, 21, "")]),
    ("step-caps.svg", "Step: capacitors C1 and C2 (stripe = − to the GND hole)",
     [(6, 4, 6, 6, "C1"), (6, 14, 6, 16, "C2")]),
    ("step-pigtails.svg", "Step: motor pigtails and the power-in pigtail (zip-tie for strain relief)",
     [(5, 7, 5, 10, "M1 + M2"), (5, 17, 5, 20, "M3 + M4"), (2, 25, 4, 25, "power in")]),
]
BOTTOM_STEPS = [
    ("step-wires-ribbon.svg", "Wires 1-14: the two 'ribbons' (driver control pin → ESP32 pin, same row)", set(range(1, 15))),
    ("step-wires-3v3.svg", "Wires 15-16: 3V3 from the ESP32 to each driver's VCC (white)", {15, 16}),
    ("step-wires-power.svg", "Wires 17-26: VM (red) and GND (black)", set(range(17, 27))),
    ("step-wires-motors.svg", "Wires 27-34: motor pigtails → driver outputs", set(range(27, 35))),
]


# --- HTML guide -------------------------------------------------------------------------------

def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


VIDEOS = [
    ("Crimping with an SN-28B ratchet crimper", "https://player.vimeo.com/video/449552756", "https://vimeo.com/449552756"),
    ("Using perfboard (soldering basics)", "https://www.youtube-nocookie.com/embed/l9Kbr8cPqOE", "https://youtu.be/l9Kbr8cPqOE"),
]
LINKS = [
    ("Core Electronics: How to make and crimp custom connectors (guide with video)", "https://core-electronics.com.au/guides/prototyping/how-to-crimp-custom-connectors/"),
    ("IWISS SN-28B: product page with demo video", "https://iwiss.com/products/sn-28b-dupont-crimping-tool-for-3-96mm-2-54mm-2-5mm-pitch-dupont-jst-xh-vh-connectors-awg-18-to-28"),
    ("Making a custom JST cable", "https://zbotic.in/how-to-make-custom-jst-cable-at-home-tools-and-steps/"),
    ("Instructables: Using Perfboard", "https://instructables.com/Using-Perfboard"),
    ("Adafruit Collin's Lab: Breadboards & Perfboards", "https://learn.adafruit.com/collins-lab-breadboards-and-perfboards"),
    ("Pololu: Dealing with motor noise", "https://www.pololu.com/docs/0J15/9"),
]

# Steps: (id, title, image, [instructions], [(kind, text)] callouts). "!" marks a must-check item.
STEPS = [
    ("overview", "What you are building", "system.svg", [
        "A perfboard <b>carrier</b> holds the ESP32 and both TB6612FNG drivers in plug-in sockets.",
        "The four motors and the motor power bank connect with latching JST-SM plugs.",
        "The ESP32 has its own power bank on its USB-C port; the grounds join on the carrier.",
        "Every wire on the carrier is in the generated wire list, which is proven to match the KiCad schematic.",
    ], [("tip", "Work top to bottom. Tick each box as you go; ticks are saved in this browser.")]),
    ("parts", "1. Gather the parts", "parts.svg", [
        "1 × ESP32-S3 CORE board, 2 × TB6612FNG modules, 4 × TT motors.",
        "2 × 5 V USB power banks (motors, ESP32), 1 × USB-C cable, 1 × USB-A cable you can cut.",
        "4 × JST-SM 2-pin pairs, 1 × JST-SM 3-pin pair, about 20 crimp terminals (pins and sockets).",
        "2 × electrolytic 220-470 µF ≥ 10 V (C1, C2), 4 × ceramic 100 nF marked <code>104</code>.",
        "Perfboard ≥ 35 × 27 holes (about 9 × 7 cm), female header strips: 2 × 22-pin, 4 × 8-pin.",
        "Solid 22 AWG wire in several colours (about 2 m), stranded 22-24 AWG in two colours (about 2 m), heat-shrink, zip ties.",
    ], [("note", "Tools: soldering iron, solder, SN-28B type crimper, wire stripper, flush cutters, multimeter, heat gun or lighter.")]),
    ("harness", "2. Make the four motor harnesses", "harness.svg", [
        "Cut two stranded wires per motor, 15-25 cm: yellow = lead 1, green = lead 2.",
        "Solder one wire to each motor tab.",
        "Solder a <b>100 nF ceramic across the two tabs</b>, legs short, close to the motor.",
        "Heat-shrink the joints, then twist the two wires around each other.",
        "Crimp <b>pin</b> terminals on the free ends (next step) into a 2-pin <b>pin-side</b> housing; lead 1 in the same position on all four.",
        "Label each motor: FL, RL, FR, RR.",
        "Make 4 carrier pigtails, about 8 cm: <b>socket</b> terminals in a 2-pin <b>socket-side</b> housing, other ends stripped and tinned.",
    ], [("why", "The side that carries power gets the recessed sockets, so an unplugged live plug can't short. A reversed motor only spins backwards; the firmware inverts it.")]),
    ("crimp", "3. Crimp each terminal", "crimp.svg", [
        "Strip 2-2.5 mm: bare copper under the front (conductor) wings only.",
        "Terminal in the smallest die slot that closes both wing pairs, wings up.",
        "Insert the wire, squeeze until the ratchet releases.",
        "!Check: wings rolled tight, no insulation under the conductor wings, a firm tug doesn't pull it out.",
        "Push into the housing, barb toward the latch window, until it clicks. It must not pull back out.",
    ], [("video", "0"), ("tip", "A bad crimp costs a terminal; a hidden bad crimp costs an evening. Redo any doubtful one.")]),
    ("power", "4. Make the motor power cable", "power-cable.svg", [
        "Cut the USB-A cable about 30 cm from the plug; strip the outer jacket 4 cm.",
        "!Plug it into the motor power bank and <b>measure</b> which wire is +5 V (usually red). Unplug.",
        "Cut the data wires short and heat-shrink each one separately.",
        "Crimp <b>sockets</b> on red and black into a 3-pin <b>socket-side</b> housing: <b>red = 1, black = 3</b>, 2 empty. Red dot on pin 1.",
        "Make the carrier's power-in pigtail the same way with <b>pins</b> in a 3-pin <b>pin-side</b> housing: red 1, black 3.",
    ], [("why", "3 pins can't mate with a 2-pin motor plug, and the keyed housing only fits one way round.")]),
    ("dryfit", "5. Dry-fit the carrier (no solder)", "carrier-top.svg", [
        "Plug the female header strips onto the ESP32 and both drivers.",
        "ESP32: left header in column 20, right header in column 30, rows 1-22, antenna end at row 1. <b>10 holes</b> between the headers.",
        "Driver 1: power header (VM…GND) in column 10, control header (PWMA…GND) in column 16, rows 4-11. <b>6 holes</b> between the headers.",
        "Driver 2: the same columns, rows 14-21.",
        "Check the antenna sticks out past the top edge and both USB-C ports at the bottom end are reachable.",
    ], [("warn", "If your boards' header spacing is not 10 and 6 holes, stop: the layout is regenerated from one table.")]),
    ("headers", "6. Solder the header sockets", "step-headers.svg", [
        "Solder each strip <b>with its board plugged in</b>, so the sockets stay aligned.",
        "Tack one pin at each end, check the strip sits flat, then solder the rest.",
        "Remove the boards again; they stay out until the first power-up.",
    ], [("video", "1")]),
    ("caps", "7. Fit C1 and C2", "step-caps.svg", [
        "C1: + leg (long) in hole (6, 4), − leg (stripe) in (6, 6).",
        "C2: + leg in (6, 14), − leg in (6, 16).",
        "!The stripe faces the GND hole on both. Solder and trim the legs.",
    ], [("warn", "A reversed electrolytic can burst. Check twice.")]),
    ("pigtails", "8. Fit the pigtails", "step-pigtails.svg", [
        "Motor pigtails, bare ends through from the top: M1 lead 1 → (5, 7), lead 2 → (5, 8); M2 lead 1 → (5, 10), lead 2 → (5, 9).",
        "M3 lead 1 → (5, 17), lead 2 → (5, 18); M4 lead 1 → (5, 20), lead 2 → (5, 19).",
        "Power-in pigtail: red (VM) → (4, 25), black (GND) → (2, 25).",
        "Zip-tie each pigtail through two free holes, so a tug pulls the tie, not the joint.",
    ], []),
    ("ribbon", "9. Wires 1-14: the two ribbons", "step-wires-ribbon.svg", [
        "Turn the board over. Each wire is a short hop along one row: driver control pin → ESP32 left-header pin.",
        "Strip 3 mm, bend the end around the pin on the solder side, solder. Tick each wire in the list below.",
    ], [("tip", "Solid insulated wire. Bare ends must not touch a neighbouring pad: check each row with the meter later.")]),
    ("v3", "10. Wires 15-16: 3V3", "step-wires-3v3.svg", [
        "White: ESP32 L1 → driver 1 VCC, ESP32 L2 → driver 2 VCC.",
    ], []),
    ("vm", "11. Wires 17-26: VM and GND", "step-wires-power.svg", [
        "Red: C1+ → D1 VM, C2+ → D2 VM, C1+ → C2+, power-in VM → C2+.",
        "Black: C1− → D1 GND, C2− → D2 GND, C1− → C2−, power-in GND → C2−, ESP32 R1 → D1 GND, ESP32 L22 → D2 GND.",
    ], [("note", "A pin or lead can take more than one wire: wrap and solder each onto the same lead.")]),
    ("motors", "12. Wires 27-34: motors", "step-wires-motors.svg", [
        "Each motor pigtail pad → its driver output on the same row (AO1, AO2, BO2, BO1).",
    ], []),
    ("checks", "13. Checks before power", "carrier-bottom.svg", [
        "Boards out, nothing plugged in. Meter in continuity (beep) mode.",
        "!Every wire in the list beeps end to end.",
        "!Each ESP32 socket in column 20 (rows 4-10, 14-20) does <b>not</b> beep to the socket above or below.",
        "!VM (C1+) to GND (C1−): no beep (a short beep while the capacitors charge is fine).",
        "!3V3 (20, 1) to GND (20, 22): no beep. VM to 3V3: no beep.",
        "Plug the motor power bank in, boards still out: about 5 V on each driver's VM socket to GND, + on C1's + leg. Unplug.",
    ], [("warn", "Find and fix any short now. Nothing of value is plugged in yet.")]),
    ("powerup", "14. First power-up", "system.svg", [
        "Plug the ESP32 and both drivers into their sockets (orientation as in the top view). Motors and motor bank unplugged.",
        "Connect the ESP32 power bank (or the laptop for flashing) to the ESP32's COM USB-C port: the LED comes on.",
        "Tell Claude: it flashes the four-motor firmware and checks the boot log.",
        "Plug in the motors, wheels off the ground, motor power bank within reach: <b>unplugging it stops every motor</b>.",
        "Step 2 of the test plan: each test is announced and starts on your \"go\".",
    ], []),
]


def html():
    swatch = {v: k for k, v in COLOUR_NAMES.items()}
    nav = "".join(f'<a href="#{sid}">{esc(t)}</a>' for sid, t, *_ in STEPS) + '<a href="#wires">Wire list</a><a href="#refs">Videos and references</a>'
    sections = []
    for sid, title, img, items, notes in STEPS:
        lis = []
        for i, it in enumerate(items):
            if sid == "overview":
                lis.append(f'<li class="info">{it}</li>')
                continue
            must = it.startswith("!")
            it = it[1:] if must else it
            lis.append(f'<li class="{"must" if must else ""}"><label><input type="checkbox" data-k="{sid}-{i}"> <span>{it}</span></label></li>')
        call = []
        for kind, body in notes:
            if kind == "video":
                t, emb, url = VIDEOS[int(body)]
                call.append(f'<div class="video"><iframe src="{emb}" title="{esc(t)}" loading="lazy" allowfullscreen></iframe>'
                            f'<a href="{url}">{esc(t)}</a></div>')
            else:
                label = {"tip": "Tip", "warn": "Careful", "why": "Why", "note": "Note"}[kind]
                call.append(f'<div class="call {kind}"><b>{label}:</b> {body}</div>')
        sections.append(f'''<section id="{sid}" class="step">
  <div class="text"><h2>{esc(title)}</h2><ol class="todo">{"".join(lis)}</ol>{"".join(call)}</div>
  <figure><a href="img/{img}" target="_blank"><img src="img/{img}" alt="{esc(title)}"></a><figcaption>click to enlarge</figcaption></figure>
</section>''')
    rows = []
    groups = {1: "Ribbon 1 (driver 1)", 8: "Ribbon 2 (driver 2)", 15: "3V3", 17: "VM and GND", 27: "Motors"}
    for n, (a, b, colour, _) in enumerate(WIRES, 1):
        if n in groups:
            rows.append(f'<tr class="grp"><td colspan="5">{groups[n]}</td></tr>')
        rows.append(f'<tr><td><input type="checkbox" data-k="w{n}"></td><td><b>{n}</b></td>'
                    f'<td><span class="sw" style="background:{colour}"></span>{COLOUR_NAMES[colour]}</td>'
                    f'<td>{esc(pin_name(*a))} <span class="hole">{PINS[a]}</span></td>'
                    f'<td>{esc(pin_name(*b))} <span class="hole">{PINS[b]}</span></td></tr>')
    refs = "".join(f'<li><a href="{u}">{esc(t)}</a></li>' for t, u in LINKS)
    page = f'''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Rovi M4: build the four-motor carrier</title>
<style>
:root {{ --accent:#d4570f; --ink:#1f2328; --muted:#59636e; --line:#d8dee4; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; font:16px/1.55 system-ui,-apple-system,"Segoe UI",Helvetica,Arial,sans-serif; color:var(--ink); background:#f6f8fa; }}
header {{ background:#1f2328; color:#fff; padding:18px 28px; }}
header h1 {{ margin:0; font-size:24px; }} header p {{ margin:4px 0 0; color:#c9d1d9; }}
.wrap {{ display:grid; grid-template-columns:240px 1fr; gap:24px; max-width:1500px; margin:0 auto; padding:20px; }}
nav {{ position:sticky; top:12px; align-self:start; background:#fff; border:1px solid var(--line); border-radius:10px; padding:10px; max-height:calc(100vh - 24px); overflow:auto; }}
nav a {{ display:block; padding:5px 8px; color:var(--ink); text-decoration:none; border-radius:6px; font-size:14px; }}
nav a:hover {{ background:#f0f3f6; }} nav a.done {{ color:#1a7f37; }} nav a.done::before {{ content:"✔ "; }}
.progress {{ font-size:13px; color:var(--muted); padding:6px 8px 10px; border-bottom:1px solid var(--line); margin-bottom:6px; }}
.bar {{ height:6px; background:#e6e9ec; border-radius:3px; margin-top:6px; }} .bar i {{ display:block; height:6px; background:#1a7f37; border-radius:3px; width:0; }}
.step {{ display:grid; grid-template-columns:minmax(320px,1fr) minmax(360px,1.35fr); gap:24px; background:#fff; border:1px solid var(--line);
        border-radius:12px; padding:20px 22px; margin-bottom:20px; scroll-margin-top:12px; }}
.step h2 {{ margin:0 0 10px; font-size:21px; }}
ol.todo {{ list-style:none; padding:0; margin:0 0 12px; }}
ol.todo li {{ padding:6px 0; border-bottom:1px dashed #eceff2; }}
ol.todo li.info {{ padding-left:22px; position:relative; }} ol.todo li.info::before {{ content:"•"; position:absolute; left:6px; color:var(--accent); }}
ol.todo li.must span::before {{ content:"MUST CHECK  "; font-size:11px; font-weight:700; color:#fff; background:#cf222e; border-radius:4px; padding:1px 5px; margin-right:6px; }}
label {{ display:flex; gap:10px; align-items:flex-start; cursor:pointer; }}
input[type=checkbox] {{ width:18px; height:18px; margin-top:3px; accent-color:#1a7f37; flex:none; }}
li:has(input:checked) span {{ color:#8c959f; text-decoration:line-through; }}
figure {{ margin:0; }} figure img {{ width:100%; border:1px solid var(--line); border-radius:8px; background:#fff; }}
figcaption {{ font-size:12px; color:var(--muted); text-align:right; }}
.call {{ border-left:4px solid; border-radius:6px; padding:8px 12px; margin:8px 0; font-size:15px; }}
.call.tip {{ border-color:#1f6feb; background:#ddf4ff; }} .call.warn {{ border-color:#cf222e; background:#ffebe9; }}
.call.why {{ border-color:#8250df; background:#fbefff; }} .call.note {{ border-color:#9a6700; background:#fff8c5; }}
.video iframe {{ width:100%; aspect-ratio:16/9; border:0; border-radius:8px; }} .video a {{ font-size:13px; }}
table {{ width:100%; border-collapse:collapse; background:#fff; font-size:14px; }}
th, td {{ border-bottom:1px solid var(--line); padding:6px 8px; text-align:left; vertical-align:top; }}
tr.grp td {{ background:#f0f3f6; font-weight:700; }} tr:has(input:checked) td {{ color:#8c959f; }}
.sw {{ display:inline-block; width:14px; height:14px; border:1px solid #888; border-radius:3px; margin-right:6px; vertical-align:-2px; }}
.hole {{ color:var(--muted); font-family:ui-monospace,monospace; font-size:12px; }}
.card {{ background:#fff; border:1px solid var(--line); border-radius:12px; padding:20px 22px; margin-bottom:20px; }}
button {{ font:inherit; padding:6px 12px; border-radius:6px; border:1px solid var(--line); background:#fff; cursor:pointer; }}
@media (max-width:1000px) {{ .wrap {{ grid-template-columns:1fr; }} nav {{ position:static; }} .step {{ grid-template-columns:1fr; }} }}
@media print {{ nav, header p, .video, button {{ display:none; }} .wrap {{ display:block; }} .step {{ break-inside:avoid; }} }}
</style></head><body>
<header><h1>Rovi M4: build the four-motor carrier</h1>
<p>From loose parts to a soldered carrier with latching plugs. Generated by <code>gen_build_guide.py</code>; the board is checked against the KiCad schematic (25 nets, 34 wires, 0 problems).</p></header>
<div class="wrap">
<nav><div class="progress"><span id="pct">0 %</span> done<div class="bar"><i id="barfill"></i></div></div>{nav}
<div style="padding:8px"><button onclick="if(confirm('Clear all ticks?')){{localStorage.removeItem(KEY);location.reload()}}">Reset ticks</button></div></nav>
<main>
{"".join(sections)}
<section id="wires" class="card"><h2>Wire list (34 wires, solder side)</h2>
<p>Holes are (column, row) in the top-view numbering. The numbers match the bottom-view drawings. Strip 3 mm, wrap the end around the pin or lead on the solder side, solder, then tick.</p>
<table><thead><tr><th></th><th>#</th><th>Colour</th><th>From</th><th>To</th></tr></thead><tbody>{"".join(rows)}</tbody></table></section>
<section id="refs" class="card"><h2>Videos and references</h2><ul>{refs}</ul>
<p>Project files: <a href="../WIRING.md">wiring table</a>, <a href="../wiring.kicad_sch">KiCad schematic</a>, <a href="../TESTPLAN.md">test plan</a>, <a href="wire-list.md">wire list (Markdown)</a>.</p></section>
</main></div>
<script>
const KEY = "rovi-m4-build";
const state = JSON.parse(localStorage.getItem(KEY) || "{{}}");
const boxes = [...document.querySelectorAll("input[data-k]")];
function update() {{
  const done = boxes.filter(b => b.checked).length;
  const pct = Math.round(100 * done / boxes.length);
  document.getElementById("pct").textContent = pct + " % (" + done + "/" + boxes.length + ")";
  document.getElementById("barfill").style.width = pct + "%";
  document.querySelectorAll("section[id]").forEach(s => {{
    const bs = [...s.querySelectorAll("input[data-k]")];
    const a = document.querySelector('nav a[href="#' + s.id + '"]');
    if (a && bs.length) a.classList.toggle("done", bs.every(b => b.checked));
  }});
}}
boxes.forEach(b => {{ b.checked = !!state[b.dataset.k];
  b.addEventListener("change", () => {{ state[b.dataset.k] = b.checked; localStorage.setItem(KEY, JSON.stringify(state)); update(); }}); }});
update();
</script></body></html>
'''
    with open(os.path.join(HERE, "index.html"), "w") as fh:
        fh.write(page)


def main():
    os.makedirs(IMG, exist_ok=True)
    wire_list()
    top_view()
    bottom_view()
    for name, title, boxes in TOP_STEPS:
        top_view(name, boxes, title)
    for name, title, only in BOTTOM_STEPS:
        bottom_view(name, only, title)
    power_cable()
    html()
    parts()
    crimp()
    harness()
    power()
    print(f"wrote {len(os.listdir(IMG))} images to {IMG}")


if __name__ == "__main__":
    ok = check()
    main()
    if "--check" in sys.argv and not ok:
        sys.exit(1)
