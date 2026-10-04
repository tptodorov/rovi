# Milestones

## M3: direct Wi-Fi and shared command ingress

**Status:** implemented and software-verified — physical acceptance pending (2026-10-04)
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

- [ ] Direct Wi-Fi client can issue every defined car command.
- [ ] BLE and Wi-Fi feed the same device-command processing path.
- [ ] Per-source ordering and cross-source arbitration are documented.
- [ ] Queue-full, stale-input, stop-priority, disconnect, and reconnect behavior are observed and recorded.
- [ ] Watchdog refresh and timeout behavior are independent of queue backlog.
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
missing evidence. Physical acceptance is pending for every item above.
