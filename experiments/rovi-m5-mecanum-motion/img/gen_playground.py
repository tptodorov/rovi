#!/usr/bin/env python3
"""Draws playground.svg: the M5 floor test area, 1 x 1 m, to scale (1 m = 600 px)."""
import math
import os

M = 600            # px per metre
OX, OY = 150, 210  # floor origin (top-left corner of the area)
S = 1.0            # area side in metres
out = []


def t(x, y, s, size=15, anchor="middle", weight="normal", fill="#222"):
    out.append(f'<text x="{x:.0f}" y="{y:.0f}" font-size="{size}" text-anchor="{anchor}" '
               f'font-weight="{weight}" fill="{fill}">{s}</text>')


def X(m):
    return OX + m * M


def Y(m):
    return OY + m * M


def line(x1, y1, x2, y2, col, w=3, dash=""):
    d = f' stroke-dasharray="{dash}"' if dash else ""
    out.append(f'<line x1="{X(x1):.0f}" y1="{Y(y1):.0f}" x2="{X(x2):.0f}" y2="{Y(y2):.0f}" stroke="{col}" stroke-width="{w}"{d}/>')


# margin and floor
out.append(f'<rect x="{X(-0.15)}" y="{Y(-0.15)}" width="{1.3 * M}" height="{1.3 * M}" rx="16" fill="#fff3d6" '
           f'stroke="#e0a030" stroke-width="3" stroke-dasharray="14,8"/>')
out.append(f'<rect x="{X(0)}" y="{Y(0)}" width="{S * M}" height="{S * M}" fill="#f4efe6" stroke="#8a7a60" stroke-width="2"/>')
t(X(0.5), Y(-0.19), "keep 15 cm clear around the area; be ready to catch the car", 14, fill="#a06000")
for i in range(1, 10):  # 10 cm grid
    line(i / 10, 0, i / 10, S, "#e2dbcc", 1)
    line(0, i / 10, S, i / 10, "#e2dbcc", 1)

cx, start, home = 0.5, 0.72, 0.42  # centre line x, START line y, home cross y

# centre line with 5 cm ticks from START up to the far edge
line(cx, 0.03, cx, 0.97, "#1f6feb", 4)
for i in range(1, 14):
    y = start - i * 0.05
    w = 0.025 if i % 2 else 0.05
    line(cx - w, y, cx + w, y, "#1f6feb", 2)
# START, 30 cm and 60 cm marks
for d, lab, col in ((0, "START", "#cf222e"), (0.6, "60 cm", "#1a7f37")):
    line(cx - 0.18, start - d, cx + 0.18, start - d, col, 5)
    t(X(cx + 0.2), Y(start - d) + 5, lab, 16, "start", "bold", col)

# home cross: centre the car here for strafe and rotation
hx, hy = cx, home
line(0.04, hy, 0.96, hy, "#8250df", 3, "10,6")   # strafe line
for d in (-0.3, -0.2, -0.1, 0.1, 0.2, 0.3):
    line(hx + d, hy - 0.025, hx + d, hy + 0.025, "#8250df", 4)
    t(X(hx + d), Y(hy) - 22, f"{abs(d) * 100:.0f}", 12, fill="#8250df")
out.append(f'<circle cx="{X(hx)}" cy="{Y(hy)}" r="{0.2 * M}" fill="none" stroke="#9a6700" stroke-width="3"/>')
for k in range(8):
    a = math.radians(k * 45)
    r1, r2 = 0.2 * M - 10, 0.2 * M + 10
    out.append(f'<line x1="{X(hx) + r1 * math.sin(a):.0f}" y1="{Y(hy) - r1 * math.cos(a):.0f}" '
               f'x2="{X(hx) + r2 * math.sin(a):.0f}" y2="{Y(hy) - r2 * math.cos(a):.0f}" stroke="#9a6700" stroke-width="3"/>')
out.append(f'<circle cx="{X(hx)}" cy="{Y(hy)}" r="7" fill="#9a6700"/>')
t(X(hx + 0.22), Y(hy - 0.17), "HOME: car centred here", 13, "start", "bold", "#9a6700")
t(X(hx + 0.22), Y(hy - 0.17) + 16, "for strafe (F4) and rotate (F5)", 12, "start", fill="#9a6700")
t(X(0.06), Y(hy) + 30, "strafe marks in cm", 12, "start", fill="#8250df")

# 45° guide from START for the diagonal
line(cx, start, cx - 0.4, start - 0.4, "#57606a", 2, "6,6")
t(X(cx - 0.42), Y(start - 0.42) - 6, "45° (F6)", 13, "middle", fill="#57606a")

# the car behind START, front edge on it (about 20 x 25 cm)
cw, cl = 0.20, 0.25
out.append(f'<rect x="{X(cx - cw / 2)}" y="{Y(start)}" width="{cw * M}" height="{cl * M}" rx="8" fill="#444" opacity="0.85"/>')
for dx in (-cw / 2 - 0.02, cw / 2 - 0.015):
    for dy in (0.015, cl - 0.075):
        out.append(f'<rect x="{X(cx + dx)}" y="{Y(start + dy)}" width="{0.035 * M}" height="{0.06 * M}" rx="3" fill="#f0d000" stroke="#333"/>')
out.append(f'<path d="M{X(cx)},{Y(start) - 30} l-12,20 h24 z" fill="#cf222e"/>')
t(X(cx), Y(start + cl / 2) + 6, "CAR", 15, weight="bold", fill="#fff")
t(X(cx + 0.13), Y(start + 0.21), "front edge on START", 12, "start")

# phones and laptop
def cam(x, y, l1, l2):
    out.append(f'<rect x="{x - 16}" y="{y - 26}" width="32" height="52" rx="6" fill="#222"/>')
    out.append(f'<circle cx="{x}" cy="{y - 14}" r="5" fill="#69b"/>')
    t(x, y + 46, l1, 13, "middle", "bold")
    t(x, y + 62, l2, 12)
cam(X(1.0) + 110, Y(start - 0.3), "phone, side view", "level with HOME, ~50 cm out")
cam(X(1.0) + 110, Y(0.08), "phone, overhead", "above HOME for rotation")
out.append(f'<rect x="{X(-0.15)}" y="{Y(1.0) + 125}" width="110" height="64" rx="6" fill="#9aa4ad"/>')
out.append(f'<rect x="{X(-0.15) + 8}" y="{Y(1.0) + 132}" width="94" height="46" rx="3" fill="#cfe3ff"/>')
t(X(-0.15) + 55, Y(1.0) + 208, "laptop (BLE), outside the area", 13, "middle", "bold")

# dimensions
out.append(f'<line x1="{X(0)}" y1="{Y(1.0) + 50}" x2="{X(1.0)}" y2="{Y(1.0) + 50}" stroke="#555" stroke-width="1.5" marker-start="url(#a)" marker-end="url(#a)"/>')
t(X(0.5), Y(1.0) + 44, "1 m", 14)
out.append(f'<line x1="{X(0) - 50}" y1="{Y(0)}" x2="{X(0) - 50}" y2="{Y(1.0)}" stroke="#555" stroke-width="1.5" marker-start="url(#a)" marker-end="url(#a)"/>')
t(X(0) - 60, Y(0.5), "1 m", 14, "end")

legend = [("#cf222e", "START: front edge of the car (F2, F3, F6, F7, F8)"), ("#1a7f37", "60 cm mark (far limit for forward runs)"),
          ("#1f6feb", "centre line, 5 cm ticks: distance, drift, stopping"), ("#8250df", "strafe line through HOME, marks at 10/20/30 cm (F4)"),
          ("#9a6700", "HOME and 40 cm circle, marks every 45° (F5)"), ("#57606a", "45° guide (F6)")]
for i, (c, s) in enumerate(legend):
    yy = Y(1.0) + 125 + i * 22
    out.append(f'<rect x="{X(0.33)}" y="{yy - 11}" width="26" height="8" fill="{c}"/>')
    t(X(0.33) + 36, yy - 2, s, 13, "start")

svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 1120" width="1000" height="1120" font-family="Helvetica, Arial, sans-serif">
<title>M5 floor test area, 1 x 1 m</title>
<defs><marker id="a" markerWidth="8" markerHeight="8" refX="4" refY="4" orient="auto-start-reverse"><path d="M0,0 L8,4 L0,8 z" fill="#555"/></marker></defs>
<rect width="1000" height="1120" fill="#fff"/>
<text x="20" y="40" font-size="22" font-weight="bold" fill="#222">M5 playground: 1 × 1 m floor test area (to scale, top view)</text>
<text x="20" y="66" font-size="14" fill="#555">Masking tape on a flat, hard floor. Runs are short (0.5-1 s) so the car stays inside; the first run measures the speed.</text>
{chr(10).join(out)}
</svg>
'''
with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "playground.svg"), "w") as fh:
    fh.write(svg)
