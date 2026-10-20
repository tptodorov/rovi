# Laptop UDP teleoperation

**Status:** laptop implementation accepted 2026-10-20. Todor selected “Laptop teleoperation client” in response to the post-M6 scope question, after the concrete proposal in #89 was presented. Proceed with the proposed client and laptop validation; firmware changes, flashes and motor runs remain excluded. Every board batch still needs a separate announced go.

## Why

The [vision](../../../docs/VISION.md) calls for teleoperation above the device API. M6 now provides that API, but the laptop tools are scenario/bench runners rather than an interactive product client.

## What Changes

* Add a Linux terminal client over the existing UDP protocol: explicit target, connect disarmed, explicit arm, six gentle directions, STOP and quit.
* Bound each recognized motion key to a 250 ms input window; stream zeros when input expires, and clear motion on STOP, faults or communication failure. No automatic claim/re-arm after failure.
* Reuse the tested TwistStamped, capability and Status codecs. Add deterministic policy tests, simulated UDP peers and product-simulator coverage before any separately authorized board test.
* Preserve M6’s firmware, ownership, wire codes, lease and 70% product cap. No iPhone/BLE client, ROS controller, station mode, floor/autonomous behavior or new physical qualification claim in this change.

## Capabilities

### New Capabilities

* `laptop-teleoperation`: interactive laptop input, client-side arming, bounded command streaming, status presentation and shutdown over the existing UDP API.

### Modified Capabilities

None. The client consumes [car-control-api](../../specs/car-control-api/spec.md); it does not alter that contract or [motor-control](../../specs/motor-control/spec.md).

## Impact

* Planned code: a small standard-library Python client under `apps/teleop`, a shared codec module, simulator tests and product usage instructions. No additional runtime dependency.
* Existing Python wire-codec callers retain compatibility; Rust/firmware and experimental behavior remain unchanged.
* Product implementation requires explicit proposal acceptance per [SPECIFICATION](../../../docs/SPECIFICATION.md). Each motor batch remains separately announced and authorized. M6’s [measurement deferrals](../archive/2026-10-20-add-four-wheel-api/verification.md#task-52-disposition) remain in effect.

## Scope choice

Todor selected laptop UDP teleoperation on 2026-10-20. The proposed keys, gentle defaults and bounded-input policy define this reversible first client; Q9 and Q21 remain separate future work.
