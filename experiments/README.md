# Hardware experiments

Each hardware milestone is an independent Cargo package, named `<milestone>-<topic>/` (e.g. `rovi-m1-wokwi/`). "Independent" means:

* its own `Cargo.toml`/`Cargo.lock` and `target/` build directory (gitignored via `/experiments/*/target/`)
* its own `README.md` (build/flash/simulate instructions), `MILESTONE.md` (goal, deliverables, test plan, acceptance) and, where relevant, `WIRING.md`
* the crate name matches the directory name, so `cargo build`/`espflash`/Wokwi paths stay predictable from the directory alone

Milestones don't share code with each other or with the Python product (`legacy-python/`) — each starts fresh from the HAL and drivers it needs. That changes once a milestone proves the ESP32-S3 platform works on real hardware: see [`../docs/VISION.md`](../docs/VISION.md) for what comes after (a shared low-level 4-wheel API, developed as a spec-driven product rather than another experiment).

After bench-testing a milestone, add a `## Results` section to its `MILESTONE.md` and update its status below.

## Milestones

* 🚧 [`rovi-m1-wokwi/`](rovi-m1-wokwi/) — ESP32-S3-N16R8 + one TB6612FNG channel, single-motor bring-up. Wokwi-simulated GPIO/PWM timing only so far; real hardware bring-up not yet started (see its `MILESTONE.md`).
