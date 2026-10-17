# M6 implementation design

## Context

See [proposal](proposal.md) for scope and evidence. M8 has a host-tested `no_std` core plus separate firmware and simulator packages; M4 added a pure motor mapping function that currently imports the arbiter state. The working firmware remains our bench reference. This change extracts that behavior and applies the reviewed corrections, with no automatic flash or measurement sweep.

## Goals / Non-Goals

**Goal:** independent motor control, one device contract for all transports, explicit validation/fault boundaries and repeatable conformance tests. **Non-goals:** a new HAL framework, closed-loop velocity control, runtime tuning UI or new physical qualification claims. The current geometry and accepted M5 references remain the inputs; no gain fitting is planned.

## Decisions

### Package boundaries

```text
Cargo.toml                      workspace: reusable no_std crates only
crates/rovi-core/                codecs, arbiter, geometry, graph, device
crates/rovi-motor-control/       configuration, mapping, enable/fault state
crates/rovi-drive/               coordinator: device state → motor control, fault → device stop
apps/simcar/                    separate workspace/lockfile; host adapters
firmware/                      separate workspace/lockfile; ESP32 adapters
experiments/                   retained standalone bench reference
```

Exclude `apps/simcar`, `firmware` and the existing experimental packages from the root workspace. M8 separated host ROS dependencies and ESP dependencies because their crypto dependency resolutions conflicted; M6 will preserve that separation rather than solve unrelated dependency upgrades. Product applications depend on the reusable crates by path. Keep the pinned zenoh fork and current target/toolchain settings initially.

Copy the working core and tests into the product crates, preserving source history through the extraction commit. Experiments stay independently buildable and frozen as evidence; subsequent product fixes live in the product crates. This bounded duplication preserves rollback and avoids silently changing the firmware used to obtain old results. After product acceptance, archive the experiments as references rather than maintaining two active implementations.

`rovi-core` returns normalized wheel demands and state effects. It does not import the motor driver or HAL. `rovi-motor-control` has no dependency on the arbiter, kinematics or transports. The coordinator, `rovi-drive` (2026-10-17), connects them; it depends on both, so the integration and fault-injection tests run on the laptop, which a coordinator inside the ESP32-only firmware would not allow. A small output boundary (`rovi_motor_control::Driver`: STBY per driver, pins per bridge) exposes standby and bridge updates to the ESP implementation or a recording test double; no generic robotics framework is needed. All three crates build without `std`.

`Drive::tick(&mut Device, now)` runs every control tick: the device's timers first, then the motors. A change of the arbiter's `arm_count` is the only enable operation, so a STOP and re-ARM between two ticks still drops STBY first; while armed it updates the bridges, otherwise it puts both drivers in standby. It returns the device `Effects` and the tick's motor fault as an `events::Fault` (`ev=error src=motor_*`); `Drive::motor().fault()` gives the typed detail.

### Configuration and fault boundary

Validate immutable device and motor profiles before accepting control. Canonicalize advertised floating values through f32 and use those values for limit comparisons, so a limit read over BLE is actually accepted. Geometry normalization remains 40 rad/s and reference limits 1/1/3 in the named experimental profile, not measured physical maxima. The product motor profile is minimum duty 40%, deadband 0.02 and maximum 70% (O1); the experiment firmware keeps its 100% cap as the comparison reference. No profile is labelled production-qualified merely because it compiles.

Motor operations are explicit enable with a complete vector, update while enabled, and standby. Validate the full vector before any drive output. A bad vector disables and latches the motor layer; ordinary updates cannot clear it. Prepare all bridge states while standby is inactive before enabling. Preserve 20 kHz PWM and the separate standby pins. Coast uses the datasheet pin state even when the *drive* duty cap is less than 100%.

On a motor validation failure, the coordinator immediately requests device out-of-limits stop, records the typed internal fault, and keeps both drivers disabled. This reuses existing status codes rather than introducing an incompatible reason enum. Only a fresh accepted setpoint followed by explicit ARM can call motor enable again; checking `state == Armed` on every tick is not an enable operation. Invalid motor configuration cannot be recovered through remote ARM; replace the configuration while disabled or reboot with a valid build. Invalid device configuration fails control initialization entirely.

### Input processing and ownership

Process due deadlines before input; then validate envelope/identity, payload, ordering and limits before committing state. An initial ROS claim requires a known publisher and a finite, planar, supported-frame, in-limits setpoint. Out-of-limits input from an existing owner raises a stop; malformed, stale or non-owner input does not mutate the lease or sequence history.

Use wrapping u32 ordering for BLE/UDP. ROS keeps full i64 counters per publisher GID, including separate arm and setpoint streams; compare only within the same stream, then order valid samples from different owner publishers by receipt. Keep bounded graph identity and session ordering tables (16 entries each initially). Graph removal invalidates identity, but retain all accepted owner GID high-water marks until session release; unrelated graph churn cannot reset replay checks. Drop input from additional owner publishers if no ordering slot remains. Unknown publishers cannot drive until announced again. A live publisher must not reset its sequence under an unchanged GID; it must create a new publisher. ROS sequence values are not narrowed to u32.

UDP session allocation uses a boot-randomized nonzero u32 seed and increments per new claim, skipping zero; a current owner's repeated HELLO is idempotent. BLE callbacks carry a local connection generation to distinguish reused handles. Clear queued inputs on release and adapter reconnect. ROS has no Rovi session ID in its payload: these measures isolate local queues and known publisher lifetimes, but do not promise authenticated replay prevention across router/client restarts. Security remains the trusted local-link baseline in ADR-0008.

BLE disconnect and UDP BYE release their owning session. A detected ROS router disconnect does the same; publisher silence or missing liveliness triggers the independent lease path when fresh setpoints cease. Reconnect rebuilds graph state and declarations before samples can claim again. Wi-Fi stays up for both UDP and ROS; the arbiter performs their mutual exclusion.

### ROS capabilities and status

Use shared typed capabilities/status snapshots; adapters serialize the same values. Preserve existing BLE/UDP frame layouts and numeric status codes. Implement persistent last-control-error state for every transport, not only BLE. BLE refreshes its readable value synchronously with changes; UDP sends its existing STATUS frame on owner control replies and stop transitions. Release leaves the externally readable state open; owner-lost is also recorded as a stop event, not a persistent stopped state with no owner.

ROS uses bounded CDR strings and the exact key/units contract in the spec. Both ROS adapters compare the shared `Status` snapshot every 10 ms and publish observed changes plus a 900 ms heartbeat; reconnect resets the publication schedule. `session=0` denotes no owner. BLE refreshes the GATT value before reads and after control/setpoint writes, then notifies changes. UDP adapters poll for pending owner stop notifications even without incoming datagrams; unsolicited STATUS uses sequence 0, direct replies echo the request sequence. No measured velocity field is added.

O9 closed on 2026-10-17: the pinned ordinary publisher did not serve retained history to a late Jazzy transient-local subscriber; the same subscriber received its later live sample ([evidence](../../../apps/simcar/scenarios/evidence/2026-10-17-capabilities/README.md)). Use the approved reliable/volatile/depth-1 fallback. `capabilities::Publication` publishes every 900 ms (headroom under one second) and on observed session/owner changes; each new router connection starts a fresh schedule. Both adapters poll it every 10 ms and serialize `Device::capabilities_snapshot()` through the same bounded CDR encoder. Controllers subscribe with volatile durability. Transport disconnection can delay delivery; publication schedules are not guarantees during packet loss or an unavailable router.

### Compatibility changes to verify

Compared with M8, intentionally correct per-wheel coast, validate configuration, latch internal motor faults, support u32 rollover and full ROS per-publisher ordering, reject unsupported frames, and process deadlines before commands. Preserve ROS auto-arm only for claim/lease-expiry and explicit ARM after STOP/out-of-limits. Add the already-agreed ROS capabilities/status content and consistent owner error persistence. These are contract improvements, not claims that extraction is behavior-identical. Client golden vectors retain version 1 layouts; existing client tools remain the bench interface.

## Risks / Trade-offs

* **Coast differs from the experiment's all-low output:** reproduce the pin mismatch in a failing host test first; product physical stopping remains unqualified until bench acceptance.
* **Lower-layer fault accidentally re-enables on the next tick:** integration tests inject faults through the coordinator and exercise continued ROS streaming, explicit recovery and session release.
* **Independent ROS publishers cannot share a sequence counter:** per-GID histories plus same-node authorization; test duplicate, out-of-order and interleaved publishers.
* **Extraction breaks the target build or Python tools:** keep separate app lockfiles, run existing socket/ROS scenarios against the product sim and cross-build both firmware features.
* **Known measurements are narrow:** carry their evidence labels; do not replace normalization or claim electrical/stopping maxima from short pulses.

## Migration and acceptance

Implement in the order in [tasks](tasks.md): baseline extraction, motor seam, device conformance, ROS observability, then qualified firmware acceptance. Tests accompany each behavioral correction. Add product checks alongside existing experiment CI while the old firmware remains the rollback reference; update client/runbook executable paths when the product sim and firmware exist.

The scenario-to-test index is created before implementation and completed with actual test names/results as each task lands. Host-only scenarios stay host-only; radio, physical output, timing and thermal claims require their corresponding bench evidence. No motor run or flash is implicit in proposal acceptance.

## Deferred qualification

O1, O2, O5 and O7 are decided (2026-10-17): 70% product cap, measured geometry with approximate references, coast for a zero wheel, direct reversals within the cap. This change does not claim what was not measured:

| Not claimed | Needs |
| --- | --- |
| Start and stall current (O1) | A current monitor (INA226) or an oscilloscope with a shunt |
| Numeric STBY disable bound (O4) | A logic analyzer or oscilloscope |
| Operation above 70% (O1, O7) | Requalification with the equipment above |
| Coast distance and maximum speeds (O5, O2) | Tape and video; not scheduled |

They return as a new change when a requirement needs them. O9 is the laptop capability test with the fixed fallback above. No new measurement sweep is scheduled.
