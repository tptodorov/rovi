# Milestones

## M3: direct Wi-Fi and shared command ingress

**Status:** proposed — not yet implemented or bench-tested
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

### Results

Add date, firmware revision, board/client details, test conditions, measurements, and pass/fail observations after bench testing.
