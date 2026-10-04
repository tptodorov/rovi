# Hardware experiments

Each hardware milestone is planned in its own `<milestone>-<topic>/` directory (e.g. `rovi-m1-tb6612/`). A proposed milestone starts with `README.md` and `MILESTONE.md`; create its independent Cargo package when firmware implementation begins. For an implemented experiment, "independent" means:

* its own `Cargo.toml`/`Cargo.lock` and `target/` build directory (gitignored via `/experiments/*/target/`)
* its own `README.md` (build/flash instructions), `MILESTONE.md` (goal, deliverables, test plan, acceptance), and, where relevant, `WIRING.md` + a `.kicad_sch` schematic (read [`../docs/reference/kicad-authoring.md`](../docs/reference/kicad-authoring.md) before writing one)
* the crate name matches the directory name, so `cargo build`/`espflash` paths stay predictable from the directory alone
* real hardware only, per the hardware-validation policy (see root [`README.md`](../README.md#hardware-validation-policy)) — no simulator dependency

Milestones don't share code with each other; each starts fresh from the HAL and drivers it needs. That changes once product API work starts: see [`../docs/PLAN.md`](../docs/PLAN.md) for the roadmap and the root [`README.md`](../README.md#how-we-work) for the spec-driven process.

After bench-testing a milestone, add a `## Results` section to its `MILESTONE.md` and update its status below.

## Milestones

* ✅ [`rovi-m1-tb6612/`](rovi-m1-tb6612/) — ESP32-S3-N16R8 + one TB6612FNG channel, single-motor bring-up. Bench-tested successfully; see its `MILESTONE.md` results.
* ✅ [`rovi-m2-ble/`](rovi-m2-ble/) — ESP32-S3 BLE peripheral and custom GATT drive-command write. Bench-tested successfully; see its `MILESTONE.md` results.
* 🔧 [`rovi-m3-wifi-queue/`](rovi-m3-wifi-queue/) — direct Wi-Fi and shared command ingress implemented and software-verified; physical acceptance pending, motors disconnected.
* 📝 [`rovi-m4-four-motor-safety/`](rovi-m4-four-motor-safety/) — motor/driver/power qualification and hardware fail-safe; one integrated four-motor bench setup.
* 📝 [`rovi-m5-mecanum-motion/`](rovi-m5-mecanum-motion/) — mecanum mapping and feedback characterization; reuse M4's setup.
* 📝 [`rovi-m7-video-teleop/`](rovi-m7-video-teleop/) — end-to-end video and direct control; gated on choosing camera/compute hardware and reuses the M4 car setup.

M6 is the spec-driven low-level API milestone, not a hardware experiment; it is tracked in [`../docs/PLAN.md`](../docs/PLAN.md#roadmap).
