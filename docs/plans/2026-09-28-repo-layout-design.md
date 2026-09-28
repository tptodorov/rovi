# Repo layout (2026-09-28)

## Context

The repo has two tracks: the working Python product (`rovi/` + `remote/`, running on a Raspberry Pi Zero 2 W) and a new Rust `no_std` firmware track for an ESP32-S3, developed milestone-by-milestone under `experiments/`. The ESP32-S3/Rust firmware is intended to eventually become the car's primary controller, replacing the RPi/Python stack — but that hasn't been proven yet (Milestone 1 is still in progress), so this is a light-touch pass: hygiene and scaffolding now, no code moves.

## Decisions

1. **Python app relocated to `legacy-python/`.** Initially kept at root (`rovi/`, `remote/`, `roviservice.py`, `event.py`) since it was still the only working product; later moved as-is into `legacy-python/` at the user's explicit request, ahead of the milestone trigger originally planned for this (see below). Internal structure and the `event.py`/`__init__.py` pattern (lets `rovi/` and `remote/` share `ControlEvent` without a circular import) are unchanged — only the location moved. `.rtx.toml` (Python 3.11 pin) and `requirements.txt` moved with it, since they're specific to this app. `legacy-python/README.md` now holds the setup/entrypoint/control-methods documentation that used to live in the root README.
2. **The bigger reorg (`firmware/` for the ESP32-S3 track) is still deferred**, triggered by a milestone proving closed-loop driving end-to-end. Only the `legacy-python/` half of the split has happened so far — `experiments/` isn't renamed to `firmware/` yet.
3. **`experiments/` gets an explicit convention**, documented in `experiments/README.md`: each milestone is an independent Cargo package named `<milestone>-<topic>/`, with the directory name matching the crate name, and its own README/MILESTONE/WIRING docs colocated with the code they describe. No repo-wide `docs/` folder for hardware reference material — the existing per-component docs pattern is working.
4. **`experiments/m1-tb6612/` renamed to `experiments/rovi-m1-wokwi/`** (via an intermediate `rovi-m1-tb6612/`). First pass matched the directory to the Cargo package name. On review, the package/crate/directory were all renamed again to `rovi-m1-wokwi`: every commit on this experiment so far (Wokwi VCD capture, panic-on-fail, ESP-IDF app descriptor) is Wokwi CI plumbing, and the `MILESTONE.md` acceptance checklist (bench-verified wiring, flashed, motor spins) is still entirely unchecked — nothing has validated the TB6612FNG on real hardware yet, so the name shouldn't claim it has. The crate's own README now says so explicitly.
5. **Root hygiene:**
   - `tptodorov.homeservice.ovpn` removed from git tracking and added to `.gitignore` (`/*.ovpn`). It contained embedded certificate/private-key material and this repo is public — the file is still in git history and the underlying credential should be rotated separately (out of scope for a layout change; requires a destructive history rewrite the user needs to authorize).
   - `remote/README.md` deployment instructions updated to say "use your own client config" instead of naming the removed file.
   - Empty, untracked `graft/.cache/` deleted — not referenced by any config in the repo.
   - `paseo.json` (worktree tool config, same category as `.envrc`/`shell.nix`) committed rather than left untracked.

## Non-goals

- No `src/`-style restructuring of the Python app.
- No Cargo workspace across milestones — each stays a fully independent package, per the existing (and now documented) convention.
- No history rewrite or credential rotation for the leaked VPN key — flagged to the user, action is theirs.
