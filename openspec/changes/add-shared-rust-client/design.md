# Shared client design

## Context and scope

See [proposal](proposal.md) for authorization and scope. The protocol mapping is [here](../../../docs/reference/protocol.md); accepted behavior lives in the [laptop](../../specs/laptop-teleoperation/spec.md), [phone](../../specs/iphone-ble-teleoperation/spec.md) and [phone network](../../specs/iphone-network-controls/spec.md) specifications. Current claim-on-write and takeover behavior also depends on the active [home-network change](../add-home-network-mode/proposal.md). Its range check remains unverified.

Current responsibilities are split between Python `apps/client/wire.py`, `apps/teleop/{protocol,policy,runtime,ble}.py` and JavaScript `apps/iphone/{wire,transport,session,hold,physical}.js`. Wire types and validating codecs for every message come from [`rovi-protocol`](../add-rovi-protocol/design.md) ([ADR-0017](../../../docs/adr/0017-one-rust-protocol-crate.md)), which this change requires first. Supported-profile validation and client state transitions still need implementing; shared codecs alone do not remove duplicated session rules.

## Shared module and interface

Use a `no_std` Rust library under `crates/rovi-client`, depending only on `rovi-protocol` for messages and codecs, never on `rovi-core` device logic. Keep operating-system, browser and executor dependencies outside it. Wire decoding belongs in `rovi-protocol`; client acceptance policy belongs in `rovi-client`. Do not introduce a second protocol definition.

The proposed seam is a deterministic session module:

```text
profile + monotonic time + event
  → session state, next deadline, requested host actions
host adapter executes an action
  → result tagged with connection generation and operation identity
```

Events represent operator ARM/STOP/network requests, latest intent with an absolute validity deadline, lifecycle changes, received messages and completed/failed host operations. The module validates data, correlates replies, manages local permission and chooses the next operation. Outputs expose requested motion, reported state/age, mode information and confirmation outcomes for rendering. Exact Rust names are implementation choices, not a public stability promise yet.

Use fixed client profiles for the already accepted laptop and phone policies, rather than public knobs that disable safety checks. Normal operation keeps bounded pending work and latest intent only. Closing uses the existing independent best-effort zero/STOP/release attempts; it must not wait indefinitely behind a stuck normal operation. All action results are checked again against generation, intent and deadlines.

## What remains in adapters

| Adapter | Responsibilities |
| --- | --- |
| Native terminal | Keyboard batches, terminal restoration, audio/bell, monotonic clock and scheduler |
| Native UDP/BLE | Sockets or BlueZ, discovery/target pinning, GATT properties/MTU, writes, reads and notifications |
| Browser JavaScript | Picker, Web Bluetooth, permissions, page lifecycle, sensor events, DOM/accessibility and clock |
| WebAssembly binding | Convert host events/results to the shared module and return decisions; no separate session policy |

Keep existing Buttons and Physical input interpreters during the phone migration. They remain responsible for eligible UI/sensor renewal, posture, calibration, START/PAUSE and computing intent validity from independent deadlines. The session module rechecks that validity and caps every axis; stream ticks cannot turn cached intent into fresh input. Moving angle algorithms into Rust is not required for protocol/session reuse.

Prefer this seam over a codecs-only wrapper, which leaves ARM, cancellation and callback bugs duplicated. Avoid placing BlueZ, Web Bluetooth or a common async runtime inside the library: their operation and recovery semantics differ. Avoid embedding Python in the native client or making JavaScript call a localhost command server; both retain extra deployment/runtime requirements.

## Targets and migration

1. Implement wire validation and the pure session module with native trace tests. Compare against independent existing Rust/Python/JavaScript vectors; preserve canonical float values rather than round-tripping human decimal text.
2. Add a thin WebAssembly binding with pinned compatible build tooling and locally packaged assets. Replay the same profile traces on native and WebAssembly targets before migrating a client. WebAssembly load failure disables connection; there is no silent second JavaScript session engine. Existing revision-pinned asset delivery must include the module and binding atomically.
3. Introduce a native laptop application in its own app workspace. Migrate UDP first, then BLE, preserving flags, key mappings, retry/beep and cleanup behavior. The accepted Python command remains available until replacements pass their respective host and stand gates; document the eventual native invocation and rollback before promotion.
4. Move the phone's protocol/session decisions behind the binding while retaining the page, input interpreters and Web Bluetooth adapter. Repeat Node/built-page tests, Bluefy loading/lifecycle/network checks, and a separately announced stand batch before replacing its accepted release.
5. Retire superseded production implementations only after checking callers and passing migration gates. Independent protocol fixtures and Python bench/simulator helpers may remain as test tools; do not claim all Python code has disappeared. Rust ROS consumers can reuse typed messages/CDR; a new ROS controller application and changes to ROS auto-arm/identity are deferred.

Before selecting a native BLE dependency, prove it exposes the required property validation, 72-byte without-response capacity, fresh read/write outcomes, notifications and disconnect behavior on this Linux/BlueZ setup. Unsupported MTU/property evidence blocks BLE promotion; it does not justify relaxing validation. Dependency pins and browser build tooling are reviewed with that feasibility result. These adapter choices do not change the session interface or permit a client to skip its gate.

## Risks and acceptance

- Policy drift → retain different laptop/phone profiles and replay current failure cases before deleting old implementations. Preserve laptop Echo/500 ms deadlines, phone Status reads/one-second controls, 50 ms stream, 200 ms stream failure and 500 ms cleanup.
- Late outcomes or broken time conversion → tag every operation/generation; use monotonic host time, reject backward/invalid times, and exercise exact deadline equality and u32 wrap on both targets.
- Browser packaging or native BLE limitations → validate loading and real host operations before switching defaults; report bundle/startup measurements without inventing unmeasured device guarantees.
- Changed input lifecycle → keep physical interpreters and their independent freshness checks; rerun focus, permissions, sample expiry and STOP/ARM races.
- No physical proof from fake peers → preserve old releases until separately authorized stand and phone checks are recorded. Roll back a failed migration to its last accepted client/revision; no firmware rollback is needed.

This draft creates no new ADR or accepted baseline. If approved implementation establishes a durable cross-client interface, record that actual decision through the repository's ADR process. Proposal approval must also resolve #108's dependency on the active home-network contract, while preserving the explicit range deferral.
