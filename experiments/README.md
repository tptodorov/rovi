# Hardware experiments

Each hardware milestone is an independent Cargo package, named `<milestone>-<topic>/` (e.g. `rovi-m1-tb6612/`). "Independent" means:

* its own `Cargo.toml`/`Cargo.lock` and `target/` build directory (gitignored via `/experiments/*/target/`)
* its own `README.md` (build/flash instructions), `MILESTONE.md` (goal, deliverables, test plan, acceptance), and, where relevant, `WIRING.md` + a `.kicad_sch` schematic (read [`../docs/reference/kicad-authoring.md`](../docs/reference/kicad-authoring.md) before writing one)
* the crate name matches the directory name, so `cargo build`/`espflash` paths stay predictable from the directory alone
* real hardware only, per the hardware-validation policy (see root [`README.md`](../README.md#hardware-validation-policy)) — no simulator dependency

Milestones don't share code with each other or with the Python product (`legacy-python/`) — each starts fresh from the HAL and drivers it needs. That changes once a milestone proves the ESP32-S3 platform works on real hardware: see [`../docs/PLAN.md`](../docs/PLAN.md) for the roadmap and the root [`README.md`](../README.md#how-we-work) for what "spec-driven" means once that starts.

After bench-testing a milestone, add a `## Results` section to its `MILESTONE.md` and update its status below.

## Milestones

* 🚧 [`rovi-m1-tb6612/`](rovi-m1-tb6612/) — ESP32-S3-N16R8 + one TB6612FNG channel, single-motor bring-up, real hardware. Not yet bench-tested (see its `MILESTONE.md`).
