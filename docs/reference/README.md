# Hardware reference

Pinouts and specs for the actual boards received, extracted from vendor documentation — plus the vendor datasheets. One directory per component; each has a `README.md` (pin tables, notes, sources), the source image(s), and the datasheet PDF(s) with a searchable `.md` transcription next to each (`rg` these first; PDFs are authoritative).

## Chip docs vs. board docs — don't mix them

Every component here exists at up to three levels, and the docs describe **different things**. Pin numbers, names, which pins exist, ports and connectors differ between levels. Wire and code against the **board** level; use chip/module docs only for electrical limits, peripheral behaviour and register detail.

| Level | Describes | Authoritative for |
| --- | --- | --- |
| **Chip** (SoC / IC) | The silicon and its package: package pin numbers, absolute maxima, peripherals, registers | Electrical limits, timing, peripheral behaviour |
| **Module** (ESP32 only) | Chip + flash/PSRAM + antenna on a castellated module | Which GPIOs are taken by flash/PSRAM, module pin numbers |
| **Board** (breakout / dev board) | The PCB actually in hand: headers, silkscreen, USB ports, LEDs, regulators, extra ICs | What to wire to, what the ports are, what is already committed |

Each component README has a *Docs by level* table saying which file is which, and each transcribed `.md` states its level in its header. Rule of thumb: if a doc talks about package pin numbers (`pin 24 = VM1`) it is chip-level, not the board in hand — see also [ADR-0003](../adr/0003-confirm-devboard-vs-chip-before-hw-design.md).

* [`esp32-s3-n16r8/`](esp32-s3-n16r8/) — the ESP32-S3-N16R8 CORE board
* [`tb6612fng/`](tb6612fng/) — the TB6612FNG dual motor driver board
* [`kicad-authoring.md`](kicad-authoring.md) — hand-authoring `.kicad_sch` files: pin-geometry formulas, a net-merge trap that's bitten twice, a real `kicad-cli` ERC false positive, and the validation checklist. Read before writing or editing any wiring diagram.

This is reference material for the hardware platform generally (used across `experiments/` and beyond), not scoped to one milestone — see the root [README.md](../../README.md) and [`VISION.md`](../VISION.md) for how the platform layers fit together.
