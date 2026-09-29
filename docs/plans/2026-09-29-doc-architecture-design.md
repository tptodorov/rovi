# Doc architecture (2026-09-29)

One topic, one home — no file repeats what another already says:

* [`docs/VISION.md`](../VISION.md) — pure vision: what Rovi is, the layering, why. No status, no roadmap, no process.
* [`docs/PLAN.md`](../PLAN.md) — living: current state, roadmap, deferred/non-goal decisions. Update in place as milestones land; don't append history here (that's what this `docs/plans/` directory is for).
* Root [`README.md`](../../README.md) — hardware reference + "How we work" (hardware-validation policy, the two development tracks, spec structure).
* [`AGENTS.md`](../../AGENTS.md) — pure index for agents: links to the above, no restated content.

Reason: the first pass (2026-09-28/29 commits) had accumulated status and process content inside `VISION.md` and duplicated rules between `AGENTS.md` and `experiments/README.md`. Corrected per explicit feedback before writing the next design doc (M2, BLE peripheral), so it lands in the right place from the start.
