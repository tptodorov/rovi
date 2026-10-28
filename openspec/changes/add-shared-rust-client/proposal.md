# Shared Rust client

**Status: draft; planning only.** On 2026-10-28 Todor selected “Draft the shared Rust client proposal (#108)”. This authorizes this proposal, not implementation, client replacement, flashing or motor tests. Proposal acceptance must be recorded before implementation.

## Why

Rust device codecs, Python laptop clients and the JavaScript phone client repeat protocol knowledge and session rules. Changes such as network information and mode confirmation must be implemented and checked separately. [#108](https://github.com/tptodorov/rovi/issues/108) proposes one reusable client implementation while retaining the accepted operator behavior.

## What Changes

- Introduce a reusable Rust client library for deterministic session decisions on top of the shared [`rovi-protocol`](../add-rovi-protocol/proposal.md) crate, which must land first.
- Expose the same library to a native laptop client and a locally bundled WebAssembly phone adapter. Provide typed protocol building blocks usable by a future Rust ROS controller; building that controller is outside this change.
- Migrate laptop UDP, laptop BLE and then the iPhone one at a time, each behind its own host and physical acceptance gate. Keep the accepted client available until its replacement passes.
- Preserve current commands, limits, explicit manual ARM, stream/cleanup deadlines, network actions and transport-specific recovery. Keep terminal, Bluetooth, browser lifecycle and sensor collection in their platform adapters.

## Capabilities

### New Capabilities

- `client-session`: a reusable client interface for validated protocol exchange, explicit motion permission, bounded work and transport-specific session recovery across native and WebAssembly hosts.

### Modified Capabilities

None. Existing laptop, phone, network and device requirements remain the migration contract. This proposal changes their implementation, not their accepted behavior.

## Impact

Proposed `crates/rovi-client`, a native laptop application and a WebAssembly adapter; existing `apps/teleop`, `apps/client`, `apps/iphone`, protocol vectors, sim scenarios, builds and CI. Typed messages and decoders come from `rovi-protocol`; this change adds no codec of its own. No device protocol, firmware pin, provisioning, router deployment or physical range result changes.

The home-network change remains active with its range check unverified. Planning can proceed; the implementation gate must explicitly confirm which home-network contract migrations target, as #108 was deferred until after that work. This proposal does not accept or archive the home-network change.
