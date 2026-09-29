# Rovi — agent guide

One topic, one home. Read the relevant file below rather than expecting the answer here — and if you're tempted to add project explanation to this file, it almost certainly belongs in one of these instead.

* **What Rovi is, why it's layered this way**: [`docs/VISION.md`](docs/VISION.md)
* **Current status, roadmap, deferred/non-goal decisions**: [`docs/PLAN.md`](docs/PLAN.md)
* **How development work happens** (hardware-validation policy, the two process tracks, spec structure): root [`README.md`](README.md#how-we-work)
* **Historical architecture/layout decisions** (dated; don't rewrite old entries — add a new one, or update `docs/PLAN.md`/`docs/VISION.md` if reality moved on): [`docs/plans/`](docs/plans/)
* **The current working product** (Python/RPi): [`legacy-python/README.md`](legacy-python/README.md)
* **Exploratory hardware bring-up convention**: [`experiments/README.md`](experiments/README.md)

## Project structure

* [`legacy-python/`](legacy-python/) — current working product, being replaced.
* [`experiments/`](experiments/) — exploratory ESP32-S3/Rust hardware bring-up, one milestone per independent Cargo package.
* `spec/` — not created yet; will hold OpenSpec change proposals once product development starts on the low-level API.
* [`docs/`](docs/) — `VISION.md`, `PLAN.md`, `plans/` (historical decisions).
