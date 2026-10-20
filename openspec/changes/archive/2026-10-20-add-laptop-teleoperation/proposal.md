# Laptop UDP teleoperation

**Status:** implemented and accepted for supervised stand use on 2026-10-20. Todor selected the laptop scope, separately authorized the announced board batch, and confirmed all directions, stops and still phases. Archived/synced through the pinned CLI on 2026-10-20; [accepted requirements](../../../specs/laptop-teleoperation/spec.md). Each future motor batch still needs its own announced go.

## Why

The [vision](../../../../docs/VISION.md) calls for teleoperation above the device API. M6 now provides that API, but the laptop tools are scenario/bench runners rather than an interactive product client.

## What Changes

* Add a Linux terminal client over the existing UDP protocol: explicit target, connect disarmed, explicit arm, six gentle directions, STOP and quit.
* Bound each recognized motion key to a 250 ms input window; stream zeros when input expires, and clear motion on STOP, faults or communication failure. No automatic claim/re-arm after failure.
* Reuse the tested TwistStamped, capability and Status codecs. Add deterministic policy tests, simulated UDP peers and product-simulator coverage before any separately authorized board test.
* Preserve M6’s firmware, ownership, wire codes, lease and 70% product cap. No iPhone/BLE client, ROS controller, station mode, floor/autonomous behavior or new physical qualification claim in this change.

## Capabilities

### New Capabilities

* `laptop-teleoperation`: interactive laptop input, client-side arming, bounded command streaming, status presentation and shutdown over the existing UDP API.

### Modified Capabilities

None. The client consumes [car-control-api](../../../specs/car-control-api/spec.md); it does not alter that contract or [motor-control](../../../specs/motor-control/spec.md).

## Impact

* Code: a small standard-library Python client under `apps/teleop`, a shared codec module, simulator tests and product usage instructions. No additional runtime dependency.
* Existing Python wire-codec callers retain compatibility; Rust/firmware and experimental behavior remain unchanged.
* Product implementation requires explicit proposal acceptance per [SPECIFICATION](../../../../docs/SPECIFICATION.md). Each motor batch remains separately announced and authorized. M6’s [measurement deferrals](../2026-10-20-add-four-wheel-api/verification.md#task-52-disposition) remain in effect.

## Scope choice

Todor selected laptop UDP teleoperation on 2026-10-20. The proposed keys, gentle defaults and bounded-input policy define this reversible first client; Q9 and Q21 remain separate future work.

## Implementation progress

The laptop client is implemented under `apps/teleop`, using shared `apps/client`
payload codecs with the former import path preserved. Host protocol/policy,
pseudo-terminal, fake-peer and product-simulator tests are linked in the
[verification map](verification.md). CI includes the client suite. The separate
stand batch passed on 2026-10-20 with Todor’s explicit wheel observations
([evidence](../../../../firmware/bench/2026-10-20-laptop-teleop/notes.md)). The
application is accepted for this supervised stand scope; no new measurement,
firmware change, flash or loaded/floor result is claimed.
