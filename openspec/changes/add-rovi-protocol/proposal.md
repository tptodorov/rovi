# One typed Rust protocol for every participant

**Status: draft; planning only.** On 2026-10-28 Todor asked that the Rust code defining the protocol be shared between the clients and the car, so communication is strongly typed, verifiable on the sim and on the device, and shared by every participant. This authorizes this proposal, not implementation, flashing or motor tests. Proposal acceptance must be recorded before implementation.

## Why

The wire contract has one partial Rust home and three hand-written copies. `rovi-core` mixes wire types with device logic and is mostly encode-only: Status, Capabilities and Network information can be produced by the car but not decoded in Rust. Python (`apps/client/wire.py`, `apps/teleop/protocol.py`), JavaScript (`apps/iphone/wire.js`) and the conformance suite (`p[27:30]` slices in `conformance.py`) each re-implement layouts, codes and limits. A field change has to be made four times and is only caught when a test happens to read the moved byte.

## What Changes

- Extract the wire contract from `rovi-core` into a `no_std`, allocation-free `crates/rovi-protocol`: every message on BLE, UDP and ROS as a typed value with symmetric, validated `encode`/`decode`, plus identifiers (UUIDs, ports, topics, kinds), lengths, codes and the protocol version.
- Type message direction: a client can only build client-to-car messages and only decode car-to-client ones, and vice versa for the car.
- `rovi-core`, firmware and sim car use `rovi-protocol` for all wire bytes; device logic stays in `rovi-core`. Existing bytes do not change.
- Publish generated golden vectors from the crate and check them in CI, so any byte change is a reviewed diff and needs a version decision.
- Bind the same crate for the other participants: a Python extension for the laptop client, bench tools and conformance suite, and a WebAssembly module for the phone. Their hand-written codecs are removed.
- Run one conformance suite, through the shared types, against the sim car in CI and against the real car on the bench (motor power off for non-motion rules).

Session logic (ARM, deadlines, recovery) is out of scope; the shared client ([#108](https://github.com/tptodorov/rovi/issues/108), draft PR #120) builds on this crate.

## Capabilities

### New Capabilities

- `wire-protocol`: one typed, versioned definition of every car message, used by the car, sim and all clients, with golden vectors and a conformance suite runnable on sim and device.

### Modified Capabilities

- `laptop-teleoperation`: the minimum accepted lease rises from 200 ms to the single client minimum shared with the phone (lease 500 ms, claim 1000 ms, nonzero reclaim), approved by Todor on 2026-10-29.

[car-control-api](../../specs/car-control-api/spec.md) behavior is unchanged. Byte layouts are unchanged except the approved ROS network key rename (T10).

## Impact

New `crates/rovi-protocol` and binding crates; `rovi-core` wire modules move; firmware and `apps/simcar` imports; `apps/client`, `apps/teleop`, `apps/iphone/wire.js`, sim scenarios and bench scripts switch to bindings; build tooling for the Python extension (maturin via the Nix shell) and WebAssembly (pinned `wasm-bindgen`); CI jobs for vectors and binding tests. The phone build must ship the module atomically with the page. Firmware behavior and flash layout are unchanged apart from code size; no provisioning, router or range result changes. The home-network change keeps its own acceptance; this change targets its current byte contract.
