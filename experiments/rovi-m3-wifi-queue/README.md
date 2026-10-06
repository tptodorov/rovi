# Rovi M3: direct Wi-Fi and shared command ingress

**Status: hardware-validated 2026-10-04, except range/distance runs and visual LED checks;
5 of 6 acceptance items met.** Board-only ESP32-S3 firmware,
independent of M1/M2. No motor GPIO, driver, motor supply, or motor is used.
Only the onboard GPIO48 RGB LED changes: forward green, reverse red, stop off.
See [MILESTONE.md](MILESTONE.md) for bench results and outstanding work.

## Build and flash

Use the repo's `nix-shell` / `.envrc`, installed `esp` Rust toolchain and
`~/export-esp.sh`, as for M2. From this directory:

```sh
# Read a private bench WPA2 passphrase without putting it in shell history.
read -rs ROVI_WIFI_PASSWORD
export ROVI_WIFI_PASSWORD
cargo build --release --locked
cargo clippy --release --locked -- -D warnings
cargo run --release --locked
```

The passphrase must be 8–63 ASCII bytes. It is embedded in the private firmware
image; do not commit credentials, share that image, or include secrets in logs.
Missing credentials fail the firmware build; invalid passphrases stop startup
before either radio starts. There is no open-AP fallback. Disconnect **all M1 wiring**, including GPIO4–7 and power,
then power the bare [CORE board](../../docs/reference/esp32-s3-n16r8/) over USB-C.
Use the physically verified USB port; the two connectors' routing is not documented.
If needed: hold BOOT, tap RESET, release BOOT, retry flashing.

Compile-time environment settings (milliseconds; 1–60000):

| Variable | Default | Purpose |
| --- | ---: | --- |
| `ROVI_WATCHDOG_MS` | 1000 | Maximum silence per connected source |
| `ROVI_MAX_AGE_MS` | 250 | Maximum admitted deadline horizon |
| `ROVI_CONSUMER_MS` | 20 | Normal FIFO drain interval; raise for overload tests |

Safety is checked every 10 ms, independently of the FIFO drain interval, and
before each validly decoded submission. These are scheduling targets, not measured
hardware bounds. Radio scheduling, USB logging and LED writes still require bench
measurement. The radio crate enables Wi-Fi/BLE coexistence. The firmware reserves
128 KiB of internal heap, including 64 KiB reclaimed RAM; PSRAM is not required.

## Direct connection and access policy

* **Car AP only:** SSID `Rovi M3`, WPA2-Personal, 2.4 GHz channel 1, one associated
  Wi-Fi station. Keep the known passphrase private. A wrong/no passphrase must
  prevent association. An approved client is anyone holding that passphrase;
  there is no per-user identity or additional application login.
* Set the laptop/phone Wi-Fi IPv4 address manually to `192.168.4.2/24`, with no
  gateway or DNS. The board is `192.168.4.1`, TCP port `7777`. No DHCP, Internet,
  router, or mobile data is needed. Accept the OS's “no Internet” network prompt.
* One active TCP controller; other sockets are not serviced while it owns the
  connection. Disconnect it before starting another client. A silent, malformed,
  or half-open controller loses its lease and the socket is closed.
* The firmware's 2 s TCP socket timeout also closes a silent Wi-Fi controller after
  about 2 s, which counts as a disconnect. So Wi-Fi silence is bounded by
  `min(ROVI_WATCHDOG_MS, ~2000)`. BLE silence is bounded only by the watchdog.
* While it tears down a session, the board briefly refuses new TCP connections
  (`ECONNREFUSED`). The client retries for up to 1 s.
* Existing-network **station mode is excluded**: it depends on external
  infrastructure and does not prove a remote joins the car's network. AP+STA,
  phone hotspots and Wi-Fi Direct/P2P are outside this experiment; no mode
  comparison is claimed. Revisit STA if infrastructure control becomes a goal.
* BLE deliberately retains M2's unpaired, unencrypted local bench access and
  single connection. WPA2 does **not** protect BLE. Use a controlled bench;
  product authorization is outside M3. Test the Wi-Fi access policy on Wi-Fi.

## Commands and protocol v1

`src/lib.rs` owns the common representation, decoder, queue and policy. Both
adapters call `Ingress::submit`; only one consumer changes the LED. There is no
motor-control API in this experiment.

| Code | Command | Consumer effect |
| ---: | --- | --- |
| 0 | stop | Priority stop; purge all queued movement |
| 1 | forward | Green |
| 2 | reverse | Red |
| 3 | ping | Refresh source lease; no queue entry or LED change |

TCP sends an ASCII discovery line immediately on connect:
`ROVI version=1 session=… now_ms=… max_age_ms=… watchdog_ms=… commands=…`.
The Python client discovers command names/codes from this line. Unknown protocol
versions fail closed. Requests are **22 bytes**, little-endian (`<BBQIQ`):
version u8, command u8, session u64, sequence u32, expiry in board uptime ms u64.
Use a new sequence starting at 1 for every attempt, including rejected attempts.
Expiry must be strictly after receive time and at most `max_age_ms` ahead.
The client sends a calibration ping (sequence 1) on connect and anchors board
time on that ping's `rx_ms`, not on the discovery banner. Client Wi-Fi power save
delivered the banner up to ~240 ms late on the bench, which made default-horizon
commands stale. Default horizon: half the advertised age. This is not
synchronized-clock one-way radio timing. Reconnect for a fresh clock anchor
during long runs.

Each response is a newline-delimited `ACK seq=… status=… order=… rx_ms=… depth=…`
or `ERR malformed`. `Queued` acknowledges admission, **not application**.
`Full`, `Stopping`, `Stale`, `Sequence`, and `Session` reject input without
refreshing the lease. Do not retry old movement; send fresh intent with a new
sequence/deadline. A partial frame cannot refresh the watchdog.

BLE uses M2's service `d47a0001-45b2-4d19-8db0-6bd87e9c0001` and write
characteristic `d47a0002-45b2-4d19-8db0-6bd87e9c0001`, advertised as **Rovi M3**.
Write exactly one byte with response. The adapter assigns session, increasing
receive sequence and `receive + max_age_ms` expiry; malformed/unknown writes and
queue rejection return ATT `VALUE_NOT_ALLOWED`. Use one outstanding write and
discard unsent movement on disconnect. M2's one-byte format has no sender time:
BLE age is measured **from firmware admission**, not from a UI's unsent backlog.
For manual discovery, build with `ROVI_WATCHDOG_MS=10000`; for sustained traffic,
write ping `03` more frequently than that interval. Compatibility is already
proven by M2; M3 tests only coexistence and shared ingress.

## Queue and watchdog policy

* Eight FIFO slots, reject newest on full; neither adapter waits for a slot.
  Accepted movement preserves source order. One monotonically increasing
  admission order arbitrates both sources; the last dequeued direction wins.
  There is no preferred controller. Opposing commands can follow one another.
* Stop bypasses a full queue, clears both sources' queued movement and latches a
  safety event. Movement is rejected until the consumer observes that event.
  Pings and further stops remain admissible. Multiple safety causes before one
  consumer tick coalesce; the first cause is retained.
* Expired movement is discarded both at admission and dequeue. Disconnection of
  **either** source conservatively stops the whole device and purges both queues.
* Each new connection has one initial watchdog lease. Only admitted valid
  commands/pings renew that source's lease. Duplicates, old sessions, malformed,
  expired, full-queue or stopping rejections never renew it. Traffic from one
  interface cannot hide a dead controller on the other.
* Timeout invalidates the source session, clears the queue and latches stop.
  The consumer checks it before FIFO work even when FIFO drain is paused.
  Expired BLE/TCP connections are closed; reconnect and send fresh intent.
  TCP EOF and AP departure events also stop; abrupt radio loss is bounded by
  the lease when the radio has not yet reported a departure.
* Fresh TCP sessions use a boot-randomized counter; BLE gets a new internal
  session per connection. TCP buffers are discarded on close; old session
  frames cannot be replayed on a new connection. No automatic command resend.

## Client and evidence

Python 3, standard library only; connect to the AP before running. Records include
the calibration ping as `seq` 1:

```sh
python3 client.py                         # every command
python3 client.py forward reverse --repeat 100 --interval .05 > latency.jsonl
python3 client.py forward --repeat 100 --burst --interval 0 > flood.jsonl
python3 client.py forward --ttl -1         # expired before admission
python3 client.py forward --send-delay .4 # expires in transit
python3 client.py 255                     # unknown command
python3 client.py ping --version 2        # unknown protocol
python3 client.py forward --hold 2        # reports socket_closed after watchdog/abort
python3 client.py stop --reconnect-replay # old session must be rejected
```

`RX` logs source/session/sequence, command, receive/expiry times, global order,
queue depth and status. `OUT` logs dequeue time plus the original entry, or safety
cause/time/purge count. `Apply` is the shared consumer; `Stale` has no LED effect.
Compare orders for FIFO (gaps from pings/stops are expected); compute queue latency
as `OUT.at_ms - entry.received_ms`. JSONL records client monotonic send/ACK times
and RTT. Burst ACK times include time waiting to read preceding responses; use
non-burst runs for latency percentiles. Save full serial logs alongside JSONL.

Host-only checks, run from the repo root (avoid the experiment's embedded default target):

```sh
cargo +stable test --locked --manifest-path experiments/rovi-m3-wifi-queue/Cargo.toml --lib --target x86_64-unknown-linux-gnu
cargo +stable clippy --locked --manifest-path experiments/rovi-m3-wifi-queue/Cargo.toml --lib --tests --target x86_64-unknown-linux-gnu -- -D warnings
cargo +esp fmt --manifest-path experiments/rovi-m3-wifi-queue/Cargo.toml -- --check
(cd experiments/rovi-m3-wifi-queue && python3 -m unittest -v test_client.py)
```

The radio API is pinned to [esp-radio 1.0.0-beta.1](https://docs.espressif.com/projects/rust/esp-radio/1.0.0-beta.1/esp32s3/esp_radio/wifi/index.html),
with the same HAL/runtime and BLE host family as M2. Build and unit checks are
software evidence only; they do not validate radio behavior or timing.
