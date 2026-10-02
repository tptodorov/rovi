# Authoring `.kicad_sch` files by hand

Lessons from building `experiments/rovi-m1-tb6612/wiring.kicad_sch`, paid for the hard way (a calibration test file to derive the pin-geometry formulas, and two separate real net-shorts from the same mistake). Read this before touching a `.kicad_sch` file in this repo; it exists so the next session doesn't re-pay the same cost.

## Before drawing anything

Confirm whether the design targets dev boards or bare chips, and if dev boards, which specific ones — never assume (see the root [README.md](../../README.md#hardware-design-assumptions), [ADR 0003](adr/0003-confirm-devboard-vs-chip-before-hw-design.md)). Then build the symbol from that board's full pinout in [`docs/reference/`](.), not a generic chip datasheet — every physical pin present, in the real header order, pin **names** the literal silkscreen labels. Pins the design doesn't use still go in the symbol, each with a `no_connect` marker, not omitted.

## Pin geometry (empirically calibrated, do not re-derive)

A pin's `(at X Y ANGLE)` gives the **electrical tip** (where wires attach) — the visible stub is drawn from there *toward the symbol body*, not away from it:

* `angle 0` → tip is the **leftmost** point, stub extends **+X** (rightward, into the body). Use for a header whose pins should stick out to the symbol's **left**.
* `angle 180` → tip is the **rightmost** point, stub extends **-X** (leftward, into the body). Use for a header facing **right**.

Two boards facing each other across a corridor (typical layout: board A left, board B right, connected by wires in between) want board A's facing header at `angle 180` and board B's facing header at `angle 0` — they point at each other. Getting this backwards is a silent, no-error mistake: pins land facing the wrong way and every wire routes through the symbol body instead of around it.

**Y is flipped** between symbol-library space and sheet space: `sheet_y = instance_y - local_y`. A pin drawn "below" its symbol's origin in the library (`local_y` negative) ends up "below" the instance on the sheet too, but the sign flips through the subtraction — compute `local_y` as `-(row_index * pitch)` for a top-to-bottom header, not `+`.

## The label-on-wire trap (cost: two real net-shorts)

**A label attaches to whatever wire is physically under it — including a different net's wire.** Two nets sharing a coordinate get silently merged: the electrical model treats them as one net, no error, no warning at the point of the mistake. This happened twice in this file: once merging `VM` into `GND` (motor supply shorted to ground), once merging `STBY` into `AIN2` — both times because a label's `(at X Y)` happened to land on another net's routing lane, usually because two lanes' coordinates were picked independently without checking for overlap.

**Before placing a label**, check its `(x y)` doesn't lie on *any* other wire segment in the file, not just the one it's meant to annotate — a crossing lane, an elbow, another net's straight run. When wires bend through shared "lanes" (multiple nets routed as parallel verticals through a corridor, as in this file), each lane needs a distinct X (or Y) so labels can't land on the wrong one by coincidence.

**Verification that actually catches this**: ERC does not. Run `kicad-cli sch export netlist <file> -o /tmp/x.net` and read every net's pin list — that's KiCad's own connectivity resolution, the same one the GUI uses, and it's the only thing that caught both merges in this file. Treat a clean `sch erc` as necessary, never sufficient.

## `kicad-cli sch erc` false positive: `wire_dangling`

A direct wire between two real pins, with no label anywhere on it, gets `wire_dangling` flagged at whichever point is listed **first** in `(pts (xy ..) (xy ..))` — regardless of what's actually connected there. Confirmed by reversing the point order and watching the false flag move with it (isolated via a series of minimal repro files: ruled out pin angle, pin electrical type, rectangle coordinates, symbol name, instance order, and UUIDs one at a time before finding the actual cause). The fix: attach one label anywhere on every direct multi-symbol wire, even when the label isn't needed for connectivity — it also silences this false positive for the entire connected run (labels, junctions included).

## Centering on the page

Symbol/wire coordinates are chosen freely (not tied to the page), so content tends to end up anchored near wherever the first element was placed — usually not the center. Before finishing: measure the content's actual bounding box (leftmost/rightmost/topmost/bottommost coordinate across every wire, symbol, and text element), then add a uniform `(OFFSET_X, OFFSET_Y)` to every coordinate so the box's midpoint lands on the page's midpoint. A pure translation touches no connectivity — re-verify with `sch erc` and `sch export netlist` anyway, but expect no diff in the netlist.

## Validation checklist (every edit)

1. `kicad-cli sch erc <file> --severity-all` — 0 errors. Warnings about `endpoint_off_grid` and `lib_symbol_issues` (missing library, since symbols are embedded rather than pulled from an installed library) are expected and fine.
2. `kicad-cli sch export netlist <file> -o /tmp/x.net` — read every net, confirm the pin membership matches intent exactly. This is the step that catches label-on-wire merges; ERC alone won't.
3. `kicad-cli sch export pdf <file> -o /tmp/x.pdf` — render and look at it. For a close-up check of small text/labels, `pdftoppm -png -r 300` (via `nix shell nixpkgs#poppler-utils -c pdftoppm ...` if not installed) gives a much higher-resolution raster than the default PDF preview.
4. `python3 scripts/check-kicad-labels.py <file>` — mechanical check for the label-on-wire trap above; catches it before ERC or netlist export are even needed.

## Generating instead of hand-writing

For anything beyond a handful of pins, hand-writing the S-expression is error-prone (this file's ESP32-S3 symbol alone has 44 pins). Generate it with a small script instead: build coordinates from a few named anchor points and header-row tables, emit wires/labels/no-connects programmatically, and re-run the validation checklist after every generator change. The script used for this file's full-pinout rebuild was a one-off (not committed — specific to this file's exact pin usage), but the pattern (anchor points + per-row tables + a uniform centering offset) is worth reusing rather than re-inventing.
