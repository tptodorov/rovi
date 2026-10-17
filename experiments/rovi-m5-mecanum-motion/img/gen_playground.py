#!/usr/bin/env python3
"""Draws playground.svg: the M5 floor test area, to scale (1 m = 200 px)."""
import os

M = 200            # px per metre
OX, OY = 140, 150  # floor origin (top-left corner of the area)
W = H = 2.5        # area in metres
out = []


def t(x, y, s, size=15, anchor="middle", weight="normal", fill="#222"):
    out.append(f'<text x="{x:.0f}" y="{y:.0f}" font-size="{size}" text-anchor="{anchor}" font-weight="{weight}" fill="{fill}">{s}</text>')


def X(m):  # metres from the left edge
    return OX + m * M


def Y(m):  # metres from the top edge
    return OY + m * M


# floor and safety margin
out.append(f'<rect x="{X(-0.25)}" y="{Y(-0.25)}" width="{(W + 0.5) * M}" height="{(H + 0.5) * M}" rx="18" fill="#fff3d6" stroke="#e0a030" stroke-width="3" stroke-dasharray="14,8"/>')
out.append(f'<rect x="{X(0)}" y="{Y(0)}" width="{W * M}" height="{H * M}" fill="#f4efe6" stroke="#8a7a60" stroke-width="2"/>')
t(X(W / 2), Y(-0.32), "keep clear: 25 cm margin around the area, no cables or obstacles", 14, fill="#a06000")

# 50 cm grid (light)
for i in range(1, 5):
    out.append(f'<line x1="{X(i*0.5)}" y1="{Y(0)}" x2="{X(i*0.5)}" y2="{Y(H)}" stroke="#ddd5c5" stroke-width="1"/>')
    out.append(f'<line x1="{X(0)}" y1="{Y(i*0.5)}" x2="{X(W)}" y2="{Y(i*0.5)}" stroke="#ddd5c5" stroke-width="1"/>')

cx = W / 2          # centre line x
start = H - 0.3     # start line y (car drives "up" the drawing)
# centre line (drift reference)
out.append(f'<line x1="{X(cx)}" y1="{Y(0.1)}" x2="{X(cx)}" y2="{Y(H-0.05)}" stroke="#1f6feb" stroke-width="4"/>')
t(X(cx) + 10, Y(start - 1.5) + 4, "centre line, 10 cm ticks", 13, "start", fill="#1f6feb")
# start line and 1 m / 2 m marks
for d, lab, col in ((0, "START", "#cf222e"), (1.0, "1 m", "#1a7f37"), (2.0, "2 m", "#1a7f37")):
    y = Y(start - d)
    out.append(f'<line x1="{X(cx-0.45)}" y1="{y}" x2="{X(cx+0.45)}" y2="{y}" stroke="{col}" stroke-width="5"/>')
    t(X(cx + 0.5), y + 5, lab, 16, "start", "bold", col)
# 10 cm ticks along the centre line for reading distances on video
for i in range(1, 21):
    y = Y(start - i * 0.1)
    w = 0.06 if i % 5 else 0.12
    out.append(f'<line x1="{X(cx-w)}" y1="{y}" x2="{X(cx+w)}" y2="{y}" stroke="#1f6feb" stroke-width="2"/>')
# strafe line: across the car's start position, marks at ±0.5 m and ±1 m
sy = Y(start - 0.6)
out.append(f'<line x1="{X(cx-1.1)}" y1="{sy}" x2="{X(cx+1.1)}" y2="{sy}" stroke="#8250df" stroke-width="3" stroke-dasharray="10,6"/>')
for d in (-1.0, -0.5, 0.5, 1.0):
    out.append(f'<line x1="{X(cx+d)}" y1="{sy-14}" x2="{X(cx+d)}" y2="{sy+14}" stroke="#8250df" stroke-width="4"/>')
    t(X(cx + d) + (14 if d > 0 else -14), sy - 20, f"{abs(d):g} m", 13, fill="#8250df")
t(X(cx + 1.15), sy + 30, "strafe line (F4): car centred on it", 13, "end", fill="#8250df")
# rotation spot: circle with 45° marks around a second car position
rx, ry, rr = X(0.5), Y(0.6), 0.35 * M
out.append(f'<circle cx="{rx}" cy="{ry}" r="{rr}" fill="none" stroke="#9a6700" stroke-width="3"/>')
import math
for k in range(8):
    a = math.radians(k * 45)
    out.append(f'<line x1="{rx + (rr-12)*math.sin(a):.0f}" y1="{ry - (rr-12)*math.cos(a):.0f}" x2="{rx + (rr+12)*math.sin(a):.0f}" y2="{ry - (rr+12)*math.cos(a):.0f}" stroke="#9a6700" stroke-width="3"/>')
t(rx, ry + rr + 22, "rotation spot (F5)", 13, "middle", fill="#9a6700")
t(rx, ry + rr + 38, "70 cm circle, marks every 45°", 12, "middle", fill="#9a6700")
# diagonal guide from the start
out.append(f'<line x1="{X(cx)}" y1="{Y(start)}" x2="{X(cx-1.0)}" y2="{Y(start-1.0)}" stroke="#57606a" stroke-width="2" stroke-dasharray="6,6"/>')
t(X(cx - 0.42), Y(start - 0.22), "45° diagonal (F6)", 13, "end", fill="#57606a")

# the car at the start: about 25 x 20 cm, front up
cw, cl = 0.20, 0.25
out.append(f'<rect x="{X(cx-cw/2)}" y="{Y(start)}" width="{cw*M}" height="{cl*M}" rx="6" fill="#444" opacity="0.85"/>')
for dx in (-cw/2 - 0.025, cw/2 - 0.01):
    for dy in (-cl/2 + 0.01, cl/2 - 0.06):
        out.append(f'<rect x="{X(cx+dx)}" y="{Y(start)+(dy+cl/2)*M}" width="{0.035*M}" height="{0.05*M}" rx="2" fill="#f0d000" stroke="#333"/>')
out.append(f'<path d="M{X(cx)},{Y(start)-26} l-10,16 h20 z" fill="#cf222e"/>')
t(X(cx), Y(start) + cl*M/2 + 6, "CAR", 13, weight="bold", fill="#fff")
t(X(cx) + 0.3*M, Y(start) + cl*M - 4, "front edge on START,", 12, "start")
t(X(cx) + 0.3*M, Y(start) + cl*M + 12, "front pointing up", 12, "start")

# phone camera positions
def cam(x, y, lab1, lab2, anchor="middle"):
    out.append(f'<rect x="{x-16}" y="{y-26}" width="32" height="52" rx="6" fill="#222"/>')
    out.append(f'<circle cx="{x}" cy="{y-14}" r="5" fill="#69b"/>')
    t(x, y + 46, lab1, 13, anchor, "bold")
    t(x, y + 62, lab2, 12, anchor)
cam(X(W) + 95, Y(start - 1.0), "phone, side view", "level with the 1 m mark, ~1 m out")
cam(X(cx) + 1.6 * M, Y(-0.05) - 40, "phone, overhead", "for rotation: hold above the spot")

# laptop
lx, ly = X(-0.25) - 0, Y(H) + 120
out.append(f'<rect x="{lx-10}" y="{ly-20}" width="120" height="70" rx="6" fill="#9aa4ad"/>')
out.append(f'<rect x="{lx}" y="{ly-12}" width="100" height="50" rx="3" fill="#cfe3ff"/>')
t(lx + 50, ly + 72, "laptop (BLE), outside the area,", 13, "middle", "bold")
t(lx + 50, ly + 88, "within ~5 m of the car", 12)

# dimensions
def dim(x1, y1, x2, y2, lab, vertical=False):
    out.append(f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="#555" stroke-width="1.5" marker-start="url(#a)" marker-end="url(#a)"/>')
    if vertical:
        t(x1 - 12, (y1 + y2) / 2, lab, 14, "end")
    else:
        t((x1 + x2) / 2, y1 - 8, lab, 14)
dim(X(0), Y(H) + 50, X(W), Y(H) + 50, "2.5 m")
dim(X(0) - 50, Y(0), X(0) - 50, Y(H), "2.5 m", vertical=True)

# legend
lg = [("#cf222e", "start line: front edge of the car here"), ("#1a7f37", "1 m and 2 m marks: distance (F3, F7, F8)"),
      ("#1f6feb", "centre line with 10 cm ticks: drift and stopping distance"), ("#8250df", "strafe line, marks at 0.5 m and 1 m (F4)"),
      ("#9a6700", "rotation spot (F5)"), ("#57606a", "45° guide (F6)")]
for i, (c, s) in enumerate(lg):
    yy = Y(H) + 190 + i * 22
    out.append(f'<rect x="{X(0.9)}" y="{yy-11}" width="26" height="8" fill="{c}"/>')
    t(X(0.9) + 36, yy - 2, s, 13, "start")

body = "\n".join(out)
svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 980 1000" width="980" height="1000" font-family="Helvetica, Arial, sans-serif">
<title>M5 floor test area</title>
<defs><marker id="a" markerWidth="8" markerHeight="8" refX="4" refY="4" orient="auto-start-reverse"><path d="M0,0 L8,4 L0,8 z" fill="#555"/></marker></defs>
<rect width="980" height="1000" fill="#fff"/>
<text x="20" y="40" font-size="22" font-weight="bold" fill="#222">M5 playground: the floor test area (to scale, top view)</text>
<text x="20" y="66" font-size="14" fill="#555">Masking tape on a flat, hard floor. Mark once; every test starts from the same START line, car pointing up the centre line.</text>
{body}
</svg>
'''
open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "playground.svg"), "w").write(svg)
