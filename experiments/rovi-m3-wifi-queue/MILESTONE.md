# Milestones

## M3: direct Wi-Fi and shared command ingress

**Status:** hardware-validated 2026-10-04; 5 of 6 acceptance items met. Range/distance and visual LED checks not run
**Setup count:** one, ESP32-S3 board only; motor hardware disconnected

### Goal

Prove that a remote control connects directly to the car over its Wi-Fi network and that the Wi-Fi receiver and already-proven BLE receiver translate inputs into the same device-specific commands for one shared processing path.

### Risks covered

* Direct Wi-Fi connection mode, onboarding, authentication, latency, disconnect, and recovery.
* Device-specific command discovery/versioning needed by clients.
* Shared-queue ordering, stale commands, overflow/backpressure, and stop priority.
* Watchdog refresh from either interface and timeout behavior under queue load, without motor loads.

### Test plan

1. Implement the ESP32 Wi-Fi receiver and a direct client using the selected command protocol. Keep command definitions transport-independent.
2. Compare car access-point and existing-network station modes if both remain viable. Record connection steps, reconnection behavior, range conditions, and command round-trip latency for each tested mode.
3. Specify the Wi-Fi client access/authentication policy. Verify an approved client can connect and an unapproved client is handled as specified.
4. Send every defined car command from a directly connected Wi-Fi client. Use the established BLE input to send queue traffic; verify both inputs reach one consumer in the common representation. Do not repeat BLE client compatibility or command-coverage tests.
5. Interleave commands from BLE and Wi-Fi sources; include opposing commands. Record per-source enqueue/dequeue sequence and timestamps.
6. Saturate the queue and inject stale inputs. Record queue depth, dropped/rejected commands, backpressure, and stop/timeout handling. Confirm a queue backlog cannot delay the watchdog timeout event in firmware.
7. Disconnect and reconnect each transport. Check that stale commands are not replayed and that only valid commands or pings refresh the configurable watchdog.

### Acceptance

Real hardware only; no motor hardware is connected for this milestone.

- [x] Direct Wi-Fi client can issue every defined car command.
- [x] BLE and Wi-Fi feed the same device-command processing path.
- [x] Per-source ordering and cross-source arbitration are documented.
- [x] Queue-full, stale-input, stop-priority, disconnect, and reconnect behavior are observed and recorded.
- [x] Watchdog refresh and timeout behavior are independent of queue backlog.
- [ ] Wi-Fi mode, client setup, authentication/access policy, latency, and recovery behavior are specified and tested against that specification.

### Software verification (not hardware results)

2026-10-04, host `blade`; no `/dev/ttyACM*`, `/dev/ttyUSB*`, or
`/dev/serial/by-id/*` device visible. No board was flashed and no client/radio
measurements were performed. All acceptance boxes remain unchecked.

* ESP32-S3 `cargo build --release --locked`: passed with installed `esp` toolchain.
  Linker reports an RWX LOAD segment warning; no link errors. Internal heap is
  128 KiB, with 64 KiB in reclaimed RAM. Boot/heap sufficiency remains a bench check.
* Nine Rust host tests: passed. Cover interleaved/opposing commands, FIFO/full,
  stop/purge, admission/dequeue expiry, per-source watchdog, exact timeout boundary,
  disconnect/session replay and protocol decoding. The watchdog fixture proves
  a timeout at synthetic time 100 ms with eight queued commands and drain disabled;
  this is deterministic logic evidence, not a physical timing measurement.
* Two Python host tests: passed. Check discovered command framing, fragmented ACK
  stream/EOF handling and unknown-version rejection using a local socket pair.
* Host and ESP32-S3 release Clippy with warnings denied, and Rust formatting: passed.

### Remaining bench procedure

Use the [README](README.md) for connection, protocol, access and arbitration
specifications. Record date, tested git commit, board revision, client OS/app/Python
versions, channel, timing settings, distance/obstacles, USB port and confirmation
that **all motor hardware is disconnected**. Capture full serial output and client
JSONL per run; never record the passphrase. Do not check acceptance until these
observations have been reviewed. Add a Results section only after actual bench work.

1. **Boot / access / recovery.** Build/flash with a private passphrase and default
   timings. Confirm both radios start without allocation failure and LED is off.
   Join `Rovi M3`, manually configure `192.168.4.2/24`, no gateway/DNS. Wrong and
   missing WPA2 credentials must fail association; correct credentials must permit
   TCP discovery. Disconnect the approved station before the negative association
   tests (AP allows only one station). Record OS onboarding and time to connect.
   Close/reopen TCP, disable/re-enable client Wi-Fi, leave/re-enter range and reset
   the board. Each recovery must give a new session and accept fresh commands;
   no queued pre-loss movement may appear after reconnect. Record recovery times.
2. **Every Wi-Fi command.** `python3 client.py` must ACK stop, forward, reverse and
   ping. Correlate `RX` with `OUT Apply/Safety` and LED colors. Ping must leave the
   indicator unchanged. Run `python3 client.py stop --reconnect-replay`; it must
   report `status=Session` for the old frame.
3. **Latency / conditions.** Run
   `python3 client.py forward reverse --repeat 100 --interval .05 > latency.jsonl`
   first near the board, then at a recorded distance/obstacle condition. Report
   count, median, p95 and max ACK RTT and board queue latency. Working bench targets:
   near-board p95 ACK RTT <=100 ms, queue latency <=50 ms with the default drain;
   report failures without changing the targets retrospectively. No claim about
   sender-to-board one-way latency is possible from unsynchronized clocks.
4. **Shared ingress / opposing commands.** For manual BLE discovery, rebuild with
   `ROVI_WATCHDOG_MS=10000`. Connect the established BLE client to `Rovi M3` and
   the M2 write characteristic; keep it alive with `03` writes at <10 s intervals.
   Run the Wi-Fi forward/reverse loop from step 3 while sending BLE `02`, `01`,
   `02`, `00`. Verify increasing receive sequence within each session and FIFO
   global order across both sources (allow gaps for ping/stop). Both sources must
   produce the same `OUT Apply` entries. Verify stop purges movement from both,
   and the last applied opposing direction controls the LED. Do not repeat M2's
   BLE-client compatibility matrix.
5. **Full / stale / stop.** Rebuild with `ROVI_CONSUMER_MS=5000`,
   `ROVI_MAX_AGE_MS=5000`, `ROVI_WATCHDOG_MS=10000`. From Python in this directory:

   ```python
   from client import Client
   c = Client("192.168.4.1")
   frames = [c.frame("forward") for _ in range(100)]
   import time
   sent = time.monotonic_ns()
   c.socket.sendall(b"".join(frames))
   for frame in frames:
       c.receive(sent, frame)
   c.send(c.frame("stop"))
   c.close()
   ```

   Require depth 8 and `Full` rejections, then stop accepted at depth zero and
   observed within 50 ms of ingress despite the 5 s drain. Repeat with BLE `00`
   as the stop source. With no stop, keep the session alive using ping every
   0.5 s; queued entries must expire at dequeue rather than apply. Run
   `client.py forward --ttl -1` and `client.py forward --send-delay 3` against
   this configuration: both must reject stale input without refreshing the lease.
6. **Watchdog under backlog.** Keep the 5 s drain/age but restore
   `ROVI_WATCHDOG_MS=1000`. Run
   `python3 client.py forward --repeat 100 --burst --hold 2 > timeout.jsonl`.
   Require timeout safety at last admitted valid input +1000 ms (target <=50 ms
   extra), without waiting 5 s for the consumer. Repeat while flooding invalid
   command `255`, version 2, duplicates and expired frames; none may postpone
   timeout. For duplicates, send the same `c.frame("ping")` byte string repeatedly.
   For partial frames, send just its first byte and wait. With a full queue,
   fresh pings every 0.2 s must renew the lease; full movement rejections must not.
   Capture last accepted receive time, safety time, purge count and LED stop.
7. **Each transport's loss independently.** With concurrent BLE and Wi-Fi traffic,
   disconnect BLE; both queues must clear, BLE advertising must resume and new BLE
   writes must get a new session. Then keep BLE pinging while killing Wi-Fi;
   BLE must not hide the Wi-Fi timeout. Repeat with the roles reversed. Reconnect
   each, explicitly send new intent, and confirm no old session entry applies.
   Repeat the latency run during concurrent BLE traffic to characterize coexistence.

For each run record observed pass/fail, expected behavior, log filenames and any
missing evidence.

### Results (2026-10-04)

**Setup.** Firmware from `d630e26`; the firmware is unchanged by the result commit,
which only fixes `client.py`. Board: ESP32-S3 rev v0.2, 16 MB flash, via the CH343 UART
USB port (`/dev/ttyACM0`), flashed with espflash 4.4.0. Client host: `blade`,
NixOS, Linux 7.2.8, Python 3.13.15, NetworkManager 1.56.0, BlueZ 5.86, bleak 2.1.1.
Wi-Fi client: a virtual STA (`wlrovi`) on the laptop's Intel iwlwifi radio, which
**stayed on the home network (5 GHz ch 36) at the same time**. Both
networks time-share one radio, which confounds the latency figures. The board was
tethered by USB, so it was within cable length of the client, with no obstacle.
Board-only firmware configures no motor GPIO. Motor disconnection comes from the
operator brief and was not physically checked by the agent. Passphrase: a private
random 20-character string, kept outside the repo. A grep found it in no log. Logs,
JSONL and helper scripts are in [`bench/2026-10-04/`](bench/2026-10-04/). BLE client:
a scripted bleak/BlueZ driver (`tools/ble_bench.py`) instead of M2's nRF Connect;
M2's compatibility matrix was not repeated.

**Not run or not observed:**

* Visual LED colors (no camera); LED effects are inferred from `OUT` lines.
* Leaving and re-entering range, and the distance/obstacle latency run.
* Phone/OS onboarding; only Linux NetworkManager was tested.
* `--send-delay 3` in step 5: the firmware's 2 s TCP socket timeout closes the
  idle socket first (`serial-wifi-stale-senddelay.log`). Substituted
  `forward ping --ttl 1000 --send-delay 1.5`.

| Step | Result | Evidence |
| --- | --- | --- |
| 1 Boot | **Pass.** AP `Rovi M3` WPA2 ch 1 seen by scan; BLE advertising at 932 ms; no allocation failure | `serial-boot.log` |
| 1 Access | **Pass.** Wrong passphrase: 4-way handshake failed (`WRONG_KEY`). Open profile: never associated. Correct passphrase: joined in 341 ms, TCP discovery OK | `access-journal.log`, `serial-wifi-default.log` |
| 1 Recovery | **Pass.** TCP reopen, Wi-Fi toggle (×3: 305–4312 ms to associate, ≤0.6 s to fresh-session commands after association), link departure and board reset each gave a new session; no pre-loss entry applied. After reset, the client noticed AP loss in ~2 s and re-associated ~6 s after boot | `recover-*`, `serial-wifi-reset.log` |
| 2 Commands | **Fail, then pass after client fix.** First run: all four commands ACKed `Stale`. Client power save delivered the banner ~240 ms late (up to 116 ms on later fresh joins), so the client's 125 ms horizon was already past. `--reconnect-replay` was refused 3/3 (board listener gap). After the fixes, 3/3 fresh joins: stop/forward/reverse/ping ACKed, `Apply`/`Safety` matched, ping caused no output, replay got `status=Session` | `wifi-every-prefix-stale.jsonl`, `wifi-replay-prefix-refused.err`, `wifi-every-*`, `wifi-replay-*`, `serial-wifi-every.log` |
| 3 Latency, near | **Pass** (PS on, OS default). 200/200 `Queued`. ACK RTT median 42.8 / p95 65.7 / max 68.2 ms. Queue latency median 11 / p95 19 / max 19 ms | `latency-near-ps-on.jsonl` |
| 3 Latency, PS off | **Fail.** p95 135.3 / max 236.4 ms; 9/200 `Stale` at admission, 1 stale at dequeue. Likely the shared-radio confound; not repeated on a dedicated client | `latency-near-ps-off.jsonl` |
| 3 Latency, distance | **Not run** | — |
| 4 Shared ingress | **Pass.** BLE and Wi-Fi overlapped for 12 s. Per-session sequence increased strictly; global admission order increased strictly across sources; 204 `Apply` in FIFO order (199 Wi-Fi, 5 BLE). Opposing commands alternated (e.g. Wi-Fi Reverse order 44, then BLE Forward 45). BLE stop at depth 0 | `serial-shared-run.log`, `shared-*.jsonl` |
| 5 Full/stop | **Pass.** Wi-Fi burst: depth 8, 92 `Full`, stop admitted at depth 0, `purged: 8`, observed 6 ms after ingress despite the 5 s drain. BLE: 8 queued, 3 `Full`, BLE `00` purged 8 in 2 ms. Mixed: BLE `00` purged 3 Wi-Fi + 2 BLE entries | `serial-wifi-full-stop.log`, `serial-ble-C-full-stop.log`, `serial-mixed-stop.log` |
| 5 Stale | **Pass.** With pings every 0.5 s, entries dequeued as `Stale` (Wi-Fi 3/4, BLE 3/4). `--ttl -1` and in-transit delay gave `Stale` at admission | `serial-wifi-stale-*.log`, `serial-ble-D-stale.log` |
| 6 Watchdog | **Pass.** 1 s watchdog with 5 s drain/age. Timeout at last admitted input +1005 to +1009 ms with 8 queued, for: Wi-Fi burst `--hold 2`, and floods of `Full`, invalid `255`, version 2, duplicate ping (`Sequence`), expired frames and a partial frame. Same for BLE (`Full`, invalid write). Pings every 0.2 s (Wi-Fi) or ~0.29 s (BLE) kept full-queue sessions alive (12.6 s, 4 s) | `serial-timeout.log`, `serial-flood-*.log`, `serial-ble-{E,F,G,H}-*.log` |
| 7 Per-transport loss | **Pass.** BLE disconnect purged both sources' entries (3), advertising resumed, the new session's fresh intent applied. Wi-Fi link departure purged 4 (2+2) within 10 ms while BLE kept pinging. Silent Wi-Fi socket closed after 2.08 s although BLE pinged. Silent BLE timed out at +10008 ms (10 s build) while Wi-Fi pinged; a forced BLE disconnect stopped at once. No old-session entry applied after any reconnect | `serial-loss*.log`, `loss*.jsonl` |
| 7 Coexistence latency | **Pass.** With concurrent BLE: ACK RTT median 51.9 / p95 65.8 / max 75.9 ms; queue p95 18 ms | `shared-wifi.jsonl` |

**Findings:**

* **Client clock anchor (fixed, test-first).** The banner-based anchor made commands
  stale under client power save. The client now anchors on a calibration ping.
* **Reconnect race (fixed in client, test-first).** The board's single listener
  refuses connections during teardown; the client now retries for up to 1 s.
  The firmware still refuses during that window.
* **`--hold` crash (fixed, test-first).** The board aborts expired sessions with RST,
  and the client crashed on it. The client now reports `socket_closed`.
* **2 s TCP socket timeout (firmware, not changed).** It caps Wi-Fi silence at ~2 s
  even when `ROVI_WATCHDOG_MS` is larger. This is the safe direction, but the
  configured watchdog is not what governs Wi-Fi.
* **BLE initial lease.** With the default 1 s watchdog, BlueZ service discovery
  (>1 s on a first or uncached connection) lost the initial lease before the first
  write, as the README predicts. Cached reconnects fit within 1 s.

**Wi-Fi acceptance item left open:** range/recovery-by-distance, the distance latency run
and phone onboarding are untested. Latency was measured only on a shared-radio
client. Wi-Fi silence is governed by the 2 s socket timeout, not the specified watchdog.
