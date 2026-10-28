# Logging

One log standard for the sim car and the real car, so the same parser, scenarios and plots work on both. Goals: **cheap on the CPU**, **easy to emit and to parse**, **easy to extend with more properties**. Decision record: [ADR-0009](adr/0009-logfmt-events-queued-and-paced.md).

## Line format

```text
ev=<name> [key=value ...] at_ms=<n>
```

```text
ev=claimed source=Udp at_ms=5193
ev=state state=Armed wheels_pm=250,250,250,250 at_ms=6736
ev=stop reason=LeaseExpired lease_late_ms=7 at_ms=7210
```

* One event per line. Parsers use only lines with `ev=`; any other line (banners, panic output, diagnostic detail such as the reason a ROS session ended) is ignored.
* `at_ms` comes last: device uptime in ms, a `u32` that wraps every 49.7 days. A line without it was cut short and is ignored.
* Values hold no spaces. Integers only, no floats: scale to an integer and put the unit in the key (`_ms`, `_pm` for permille). Lists are comma-separated. Enums use the Rust variant name (`LeaseExpired`).
* **Growing:** add keys at the end of an event, add whole events, never rename, repurpose or remove a key. Parsers keep keys they don't know.

## Events

| `ev` | Fields | Emitted when |
| --- | --- | --- |
| `boot` | `ver`, `reset` (board only: the chip's reset cause, e.g. `ChipPowerOn`, `SysRtcWdt`, `SysBrownOut`) | Once at startup. Starts a new log segment. |
| `claimed` | `source` = `Ble`, `Udp`, `Ros` | A controller took ownership |
| `released` | - | Ownership was released |
| `stop` | `reason` = `Stop`, `LeaseExpired`, `OwnerLost`, `OutOfLimits`; `lease_late_ms` on instrumented lease stops | A safety stop |
| `state` | `state` = `Open`, `Claimed`, `Armed`, `Stopped`; `reason` (Stopped only); `wheels_pm` = FL,FR,RR,RL | The state or a wheel command changed |
| `ble` | `link` = `up`, `down`; `id`; `mtu` (up only) | BLE connected or disconnected |
| `mtu` | `id`; `mtu` | The central's ATT MTU exchange completed (once per connection). The `mtu` in `ble link=up` is the value at connect, before the exchange (23 by default) |
| `wifi` | `ap` = `on`, `off` | The Wi-Fi AP was switched |
| `ros` | `link` = `up`, `down`, `retry`, `refresh` | The zenoh session changed |
| `net` | `mode` = `Home`, `Attempting`, `Fallback`; `step` (Attempting: `Scan`, `Join`, `Dhcp`, `Router`); `reason` (Fallback: `NoSettings`, `Scan`, `Join`, `Dhcp`, `Router`, `LinkLost`, `Requested`) | The network mode changed ([ADR-0016](adr/0016-home-network-mode-with-ble-override.md)) |
| `net_step` | `step`; `result` = `Ok`, `Failed`; `took_ms` | A home-mode step finished |
| `mode_cmd` | `source`; `cmd` = `Home`, `CarNetwork`; `result` = `Ok`, `NotReady`, `Invalid` | An owner sent a mode command |
| `resolve` | `result` = `Ok`, `Failed`; `took_ms`; `ip` (0.0.0.0 on failure) | The home ROS router's host name was looked up (mDNS for `*.local`); a literal IPv4 logs nothing |
| `preempted` | `source` (of the old owner) | A higher-priority controller took the car; followed by `stop reason=OwnerLost`, `released`, `claimed` |
| `error` | `src` | A fault. `sim/carlog.py` flags it. `src`: `wifi_ap` (the AP switch failed); `motor_nonfinite`, `motor_config`, `motor_driver` (motor control refused a NaN/infinite wheel command, has an invalid configuration, or a pin write failed; the car raises an out-of-limits stop). |
| `dropped` | `n` | `n` events were lost because the ring was full. Flagged too. |

The catalog is [`crates/rovi-core/src/events.rs`](../crates/rovi-core/src/events.rs)
(`Event` and its `Display`); [`carlog.py`](../apps/simcar/scenarios/carlog.py)
has the same list, and [`test_carlog.py`](../apps/simcar/scenarios/test_carlog.py)
fails if the two differ.

`lease_late_ms` is detection time minus the actual lease deadline, captured in
the arbiter before that deadline is cleared. The deadline belongs to the last
accepted owner setpoint; stale/rejected input cannot move it. This integer
duration uses the device clock and survives queued/deferred effect delivery.
It is independent of `at_ms`, serial delivery and laptop observation time.
A value at most 50 establishes the lease-detection bound for that stop; it
does not measure driver-pin propagation or wheel stopping time. Older logs
lack this field and cannot provide this measurement. Other stop reasons omit
it, including a motor fault that replaces a lease stop.

The opt-in [motor reversal bench image](../firmware/bench/MOTOR-REVERSALS.md)
prints an explicit `BENCH ONLY motor-reversal` banner. Its `wheels_pm` values
are the injected motor-module demands (including ±1000), not the ordinary
kinematic output. The same field remains a requested command, never feedback.

## Cost

* **Emitting** (`logger::emit(Event)` on the board) copies a small plain record into a 32-entry ring. No formatting, no I/O, no waiting. A full ring drops the record and counts it.
* **Formatting and writing** happen in one task (`firmware/src/logger.rs`): integers only, one line into a stack buffer, one `write_bytes` call (one lock, not one per fragment). It then sleeps for the UART's transmit time of that line (87 µs per byte at 115200 8N1), so the 128-byte FIFO never fills and nothing busy-waits. That caps output at about 190 lines/s of 60 bytes, far above the real event rate.
* Measured: the firmware text is 13.6 KB smaller than before this change (664,481 to 650,857 bytes, no `ros` feature).
* Not measured: the CPU time per line on the board, and control-tick jitter with logging on. Both are bench items.

## Adding an event or a property

1. Add the variant or field to `Event` in `crates/rovi-core/src/events.rs`, and the matching `write!` in its `Display`. Extend the test table in the same file.
2. For a new event name, add it to `CATALOG` in `apps/simcar/scenarios/carlog.py` and to the table above.
3. Emit it: `logger::emit(...)` in the firmware, `log(..., t0)` in the sim car.
