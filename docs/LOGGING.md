# Logging

One log standard for the sim car and the real car, so the same parser, scenarios and plots work on both. Goals: **cheap on the CPU**, **easy to emit and to parse**, **easy to extend with more properties**. Decision record: [ADR-0009](adr/0009-logfmt-events-queued-and-paced.md).

## Line format

```text
ev=<name> [key=value ...] at_ms=<n>
```

```text
ev=claimed source=Udp at_ms=5193
ev=state state=Armed wheels_pm=250,250,250,250 at_ms=6736
ev=stop reason=LeaseExpired at_ms=7210
```

* One event per line. Parsers use only lines with `ev=`; any other line (banners, panic output, diagnostic detail such as the reason a ROS session ended) is ignored.
* `at_ms` comes last: device uptime in ms, a `u32` that wraps every 49.7 days. A line without it was cut short and is ignored.
* Values hold no spaces. Integers only, no floats: scale to an integer and put the unit in the key (`_ms`, `_pm` for permille). Lists are comma-separated. Enums use the Rust variant name (`LeaseExpired`).
* **Growing:** add keys at the end of an event, add whole events, never rename, repurpose or remove a key. Parsers keep keys they don't know.

## Events

| `ev` | Fields | Emitted when |
| --- | --- | --- |
| `boot` | `ver` | Once at startup. Starts a new log segment. |
| `claimed` | `source` = `Ble`, `Udp`, `Ros` | A controller took ownership |
| `released` | - | Ownership was released |
| `stop` | `reason` = `Stop`, `LeaseExpired`, `OwnerLost`, `OutOfLimits` | A safety stop |
| `state` | `state` = `Open`, `Claimed`, `Armed`, `Stopped`; `reason` (Stopped only); `wheels_pm` = FL,FR,RR,RL | The state or a wheel command changed |
| `ble` | `link` = `up`, `down`; `id`; `mtu` (up only) | BLE connected or disconnected |
| `wifi` | `ap` = `on`, `off` | The Wi-Fi AP was switched |
| `ros` | `link` = `up`, `down`, `retry` | The zenoh session changed |
| `error` | `src` | A fault. `sim/carlog.py` flags it. |
| `dropped` | `n` | `n` events were lost because the ring was full. Flagged too. |

The catalog is `src/events.rs` (`Event` and its `Display`); `sim/carlog.py` has the same list, and `sim/test_carlog.py` fails if the two differ.

## Cost

* **Emitting** (`logger::emit(Event)` on the board) copies a small plain record into a 32-entry ring. No formatting, no I/O, no waiting. A full ring drops the record and counts it.
* **Formatting and writing** happen in one task (`firmware/src/logger.rs`): integers only, one line into a stack buffer, one `write_bytes` call (one lock, not one per fragment). It then sleeps for the UART's transmit time of that line (87 µs per byte at 115200 8N1), so the 128-byte FIFO never fills and nothing busy-waits. That caps output at about 190 lines/s of 60 bytes, far above the real event rate.
* Measured: the firmware text is 13.6 KB smaller than before this change (664,481 to 650,857 bytes, no `ros` feature).
* Not measured: the CPU time per line on the board, and control-tick jitter with logging on. Both are bench items.

## Adding an event or a property

1. Add the variant or field to `Event` in `src/events.rs`, and the matching `write!` in its `Display`. Extend the test table in the same file.
2. For a new event name, add it to `CATALOG` in `sim/carlog.py` and to the table above.
3. Emit it: `logger::emit(...)` in the firmware, `log(..., t0)` in the sim car.
