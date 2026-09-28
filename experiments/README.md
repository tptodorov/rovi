# Hardware experiments

Each hardware milestone is an independent Cargo package, named `<milestone>-<topic>/` (e.g. `rovi-m1-tb6612/`). "Independent" means:

* its own `Cargo.toml`/`Cargo.lock` and `target/` build directory (gitignored via `/experiments/*/target/`)
* its own `README.md` (build/flash/simulate instructions), `MILESTONE.md` (goal, deliverables, test plan, acceptance) and, where relevant, `WIRING.md`
* the crate name matches the directory name, so `cargo build`/`espflash`/Wokwi paths stay predictable from the directory alone

Milestones don't share code with each other or with the Rovi Python product (`rovi/`, `remote/`) — each starts fresh from the HAL and drivers it needs. This track is expected to eventually graduate into a permanent `firmware/` tree once a milestone proves the ESP32-S3 can drive the car end-to-end; see the root [README.md](../README.md#hardware-milestones).

## Milestones

* [`rovi-m1-tb6612/`](rovi-m1-tb6612/) — ESP32-S3-N16R8 + one TB6612FNG channel, single-motor bring-up.
