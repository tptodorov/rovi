---
status: accepted
---

# Rovi builds on a pinned fork of zenoh-nostd; the upstream PRs are separate

[ADR-0005](0005-zenoh-nostd-ros-backend.md) chose zenoh-nostd for the ROS 2 path and said to carry patches in a pinned fork. We decided (2026-10-11) how that fork relates to the upstream pull requests:

* **Rovi depends on the fork, by commit.** `sim/simcar/Cargo.toml` and `firmware/Cargo.toml` pin `github.com/tptodorov/zenoh-nostd` at one commit on the fork's `rovi/liveliness-token` branch. It carries liveliness tokens and subscriber, sample attachments, per-priority and per-reliability sequence numbers, the zid accessor, optional `defmt` in `zenoh-embassy` and `embassy-sync` 0.8.
* **The upstream PRs do not feed Rovi.** The draft PRs to `eclipse-zenoh/zenoh-nostd` (#28 to #34) come from other branches cut from upstream `main`. They are rewrites for review, not the same commits, and they may change or be refused. Rovi does not wait for them.
* **The pinned branch is never deleted or force-pushed.** The pinned commit must stay reachable.

## Consequences

* When an upstream PR merges, repoint the pin at upstream for that change and drop it from the fork. If upstream refuses or goes quiet, nothing changes for Rovi.
* The fork's build is the one under test, so what Rovi runs can differ from what upstream accepts. Check the PR's final form before dropping a fork commit.
* Revisit if upstream stays inactive for long or its API moves incompatibly with the fork.
