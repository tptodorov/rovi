# Repo layout (2026-09-28)

## Context

The repo has two tracks: the working Python product (`rovi/` + `remote/`, running on a Raspberry Pi Zero 2 W) and a new Rust `no_std` firmware track for an ESP32-S3, developed milestone-by-milestone under `experiments/`. The ESP32-S3/Rust firmware is intended to eventually become the car's primary controller, replacing the RPi/Python stack — but that hasn't been proven yet (Milestone 1 is still in progress), so this is a light-touch pass: hygiene and scaffolding now, no code moves.

## Decisions

1. **Python app (`rovi/`, `remote/`, `roviservice.py`, `event.py`) stays exactly as-is.** It's still the only working product. The root-level `event.py`/`__init__.py` pattern exists so `rovi/` and `remote/` can share `ControlEvent` without a circular import — not worth disturbing.
2. **The bigger reorg (`firmware/` + `legacy-python/` split) is deferred**, triggered by a milestone proving closed-loop driving end-to-end on the ESP32-S3. The intent is now written down in the root README so it isn't lost.
3. **`experiments/` gets an explicit convention**, documented in `experiments/README.md`: each milestone is an independent Cargo package named `<milestone>-<topic>/`, with the directory name matching the crate name, and its own README/MILESTONE/WIRING docs colocated with the code they describe. No repo-wide `docs/` folder for hardware reference material — the existing per-component docs pattern is working.
4. **`experiments/m1-tb6612/` renamed to `experiments/rovi-m1-tb6612/`** to match the Cargo package name (`rovi-m1-tb6612`, used consistently in `Cargo.toml`, `Cargo.lock`, `wokwi.toml`, and the crate's own README title) — the directory name was the one thing inconsistent with everything else.
5. **Root hygiene:**
   - `tptodorov.homeservice.ovpn` removed from git tracking and added to `.gitignore` (`/*.ovpn`). It contained embedded certificate/private-key material and this repo is public — the file is still in git history and the underlying credential should be rotated separately (out of scope for a layout change; requires a destructive history rewrite the user needs to authorize).
   - `remote/README.md` deployment instructions updated to say "use your own client config" instead of naming the removed file.
   - Empty, untracked `graft/.cache/` deleted — not referenced by any config in the repo.
   - `paseo.json` (worktree tool config, same category as `.envrc`/`shell.nix`) committed rather than left untracked.

## Non-goals

- No `src/`-style restructuring of the Python app.
- No Cargo workspace across milestones — each stays a fully independent package, per the existing (and now documented) convention.
- No history rewrite or credential rotation for the leaked VPN key — flagged to the user, action is theirs.
