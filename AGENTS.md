@RTK.md

# Rovi — agent guide

One topic, one home. Read the relevant file below rather than expecting the answer here — and if you're tempted to add project explanation to this file, it almost certainly belongs in one of these instead.

* **What Rovi is, why it's layered this way**: [`docs/VISION.md`](docs/VISION.md)
* **Current status, roadmap, deferred/non-goal decisions**: [`docs/PLAN.md`](docs/PLAN.md)
* **How development work happens** (contribution method, hardware-validation policy, hardware design assumptions, the two process tracks, spec structure): root [`README.md`](README.md#how-we-work)
* **Decision records** — terse, one per hard-to-reverse/non-obvious/real-trade-off decision (see the `domain-modeling` skill's ADR format; most decisions don't qualify, skip recording those): [`docs/adr/`](docs/adr/), sequentially numbered, never rewritten (a reversed decision gets a new ADR marked "supersedes ADR-NNNN"). Skills that default to writing design docs elsewhere (e.g. `brainstorming`'s `docs/plans/`) write decision records here instead, in this repo.
* **Logging standard** (event lines, catalog, cost, how to add an event): [`docs/LOGGING.md`](docs/LOGGING.md)
* **Feedback loops** (how an agent sees its work run: tests, sim, car log, bench script, camera; ranked, with status): [`docs/FEEDBACK-LOOPS.md`](docs/FEEDBACK-LOOPS.md)
* **Exploratory hardware bring-up convention**: [`experiments/README.md`](experiments/README.md)
* **Hardware component pinouts/specs, verified against the actual boards received**: [`docs/reference/`](docs/reference/)

## Project structure

* [`experiments/`](experiments/) — exploratory ESP32-S3/Rust hardware bring-up, one milestone per independent Cargo package.
* `spec/` — not created yet; will hold OpenSpec change proposals once product development starts on the low-level API.
* [`docs/`](docs/) — `VISION.md`, `PLAN.md`, `adr/` (decision records), `reference/` (hardware component docs).

## Agent skills

### Issue tracker

Issues live in GitHub Issues via `gh`. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the five default triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

Use the single-context layout. See `docs/agents/domain.md`.
