#!/usr/bin/env python3
"""Catch the label-on-wrong-wire trap in a .kicad_sch file before kicad-cli
does (or worse, before it doesn't -- ERC alone never caught this).

A KiCad label attaches to whatever wire is physically under it. Two labels
with different text landing on the same connected run of wires/junctions
means two nets got silently merged -- a short, if the nets are a rail and
ground, or just wrong, otherwise. This has happened twice in this repo's
wiring.kicad_sch (VM merged into GND, then STBY into AIN2); see
docs/reference/kicad-authoring.md for the story.

Usage: python3 scripts/check-kicad-labels.py <file.kicad_sch> [...]
Exit 0: every labeled component has exactly one label text.
Exit 1: a component has 2+ different label texts (real merge) or a parse
        found nothing to check (probably a malformed path, not a clean file).
"""

import re
import sys
from dataclasses import dataclass, field


@dataclass
class Point:
    x: float
    y: float

    def key(self):
        return (round(self.x, 6), round(self.y, 6))


@dataclass
class Segment:
    p1: Point
    p2: Point

    def on_segment(self, p: Point) -> bool:
        """Axis-aligned containment test (all wires in this repo's
        generated schematics are horizontal or vertical)."""
        x1, y1, x2, y2 = self.p1.x, self.p1.y, self.p2.x, self.p2.y
        if x1 == x2 == p.x:
            return min(y1, y2) - 1e-6 <= p.y <= max(y1, y2) + 1e-6
        if y1 == y2 == p.y:
            return min(x1, x2) - 1e-6 <= p.x <= max(x1, x2) + 1e-6
        return False


class UnionFind:
    def __init__(self):
        self.parent = {}

    def find(self, k):
        self.parent.setdefault(k, k)
        while self.parent[k] != k:
            self.parent[k] = self.parent[self.parent[k]]
            k = self.parent[k]
        return k

    def union(self, a, b):
        ra, rb = self.find(a), self.find(b)
        if ra != rb:
            self.parent[ra] = rb


XY_RE = re.compile(r"\(xy\s+([-\d.]+)\s+([-\d.]+)\)")
WIRE_RE = re.compile(r"\(wire\s*\(pts((?:\s*\(xy\s+[-\d.]+\s+[-\d.]+\))+)\s*\)", re.DOTALL)
LABEL_RE = re.compile(r'\(label\s+"([^"]*)"\s*\(at\s+([-\d.]+)\s+([-\d.]+)')


def parse(text: str):
    segments: list[Segment] = []
    for m in WIRE_RE.finditer(text):
        pts = [Point(float(x), float(y)) for x, y in XY_RE.findall(m.group(1))]
        for a, b in zip(pts, pts[1:]):
            segments.append(Segment(a, b))
    labels = [(name, Point(float(x), float(y))) for name, x, y in LABEL_RE.findall(text)]
    return segments, labels


def check(path: str) -> bool:
    text = open(path).read()
    segments, labels = parse(text)
    if not segments:
        print(f"{path}: no (wire ...) elements found -- nothing to check "
              f"(parse issue, or a file with no routed nets)")
        return False

    uf = UnionFind()
    for seg in segments:
        uf.union(seg.p1.key(), seg.p2.key())

    def component_of(p: Point):
        # Exact endpoint hit first; otherwise find a segment whose line
        # contains this point and use one of *its* endpoints as the
        # component key -- this is what an on-wire (not on-endpoint) label
        # needs, and exactly the case that caused both real incidents.
        exact = p.key()
        if exact in uf.parent:
            return uf.find(exact)
        for seg in segments:
            if seg.on_segment(p):
                return uf.find(seg.p1.key())
        return None

    by_component: dict = {}
    unplaced = []
    for name, pos in labels:
        comp = component_of(pos)
        if comp is None:
            unplaced.append((name, pos))
            continue
        by_component.setdefault(comp, []).append((name, pos))

    ok = True
    for comp, entries in by_component.items():
        names = {n for n, _ in entries}
        if len(names) > 1:
            ok = False
            print(f"{path}: MERGE DETECTED -- labels {sorted(names)} all land on "
                  f"the same connected wire run:")
            for n, p in entries:
                print(f"    {n!r} at ({p.x}, {p.y})")

    if unplaced:
        print(f"{path}: {len(unplaced)} label(s) not on any wire segment "
              f"(floating label, or a diagonal/non-axis-aligned wire this "
              f"script doesn't model):")
        for n, p in unplaced:
            print(f"    {n!r} at ({p.x}, {p.y})")

    if ok and not unplaced:
        net_count = len({c for c in by_component})
        print(f"{path}: OK -- {net_count} labeled net(s), no merges detected.")
    return ok


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    all_ok = True
    for path in sys.argv[1:]:
        if not check(path):
            all_ok = False
    sys.exit(0 if all_ok else 1)
