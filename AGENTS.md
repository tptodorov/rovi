# Rovi — agent guide

Read [`docs/VISION.md`](docs/VISION.md) first — it explains what Rovi is (a layered hardware + software robotics platform, not just "a car"), why the current Python app is legacy, and why hardware milestones must be bench-verified rather than just simulated. This file is the short version plus project-specific process rules; it doesn't repeat generic engineering conventions from your own instructions.

## Project structure

- [`legacy-python/`](legacy-python/) — the current working product: Raspberry Pi Zero 2 W app, flat/car-specific, being replaced by the ESP32-S3 platform. See [`legacy-python/README.md`](legacy-python/README.md). Bug fixes here are fine; don't add new capability here that belongs in the new platform.
- [`experiments/`](experiments/) — exploratory ESP32-S3/Rust hardware bring-up, one milestone per independent Cargo package. See [`experiments/README.md`](experiments/README.md) for the convention.
- [`docs/VISION.md`](docs/VISION.md) — product vision, layering, and process model.
- [`docs/plans/`](docs/plans/) — dated records of past architecture/layout decisions (historical; don't rewrite old entries when reality moves on, add a new one or update the vision doc instead).
- `spec/` — not created yet. Will hold OpenSpec change proposals (`spec/changes/{change-id}/{proposal.md,tasks.md,specs/*.md}`) once product development starts on the low-level API. Use the `openspec-implementation` skill for that workflow when it exists.

## Rules specific to this repo

1. **A hardware milestone is not done because a simulation passed.** Wokwi (or any simulator) only proves what it actually models. If it doesn't model the IC/sensor/whatever in question, physical bench verification is required before checking off acceptance criteria. Don't mark a `MILESTONE.md` acceptance item complete from simulation results alone.
2. **Record results, not just plans.** After bench-testing a milestone, add a `## Results` section to its `MILESTONE.md` (what was observed, pass/fail, date, links to logs/photos/VCD) and update its status marker in `experiments/README.md`'s index.
3. **`experiments/` packages stay independent** — no shared Cargo workspace — until the low-level 4-wheel-drive API becomes real product work (see `docs/VISION.md`). Don't introduce shared crates or cross-milestone dependencies preemptively.
4. **Two process tracks, don't mix them**: exploratory bring-up in `experiments/` stays lightweight (README/MILESTONE/WIRING, no spec required). Product development — the low-level API and anything built on it — goes through OpenSpec proposals once that work starts. Don't retroactively write specs for `legacy-python/` or for exploratory experiments; don't skip specs once real product development begins.
5. **Don't design the low-level API around mecanum/car specifics it doesn't need.** The point of the layering is that hardware platform and low-level control API stay reusable for other vehicle types later (boat, flying device). Car-specific logic belongs in the application layer.
6. **Repo stays a monorepo for now.** Don't split the platform into a separate repo — that's deferred until a second real hardware application actually needs to consume it independently.
