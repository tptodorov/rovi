# Design

## Context

See [proposal.md](proposal.md) for why. Today `firmware/src/fw.rs` builds one embassy-net stack on the AP interface with a static `192.168.4.1/24`, and a `wifi_gate` task switches the AP off while BLE owns the car. `rovi-core`'s `Device::radio_allowed` stops BLE advertising while a Wi-Fi source owns the car; `wifi_allowed` turns Wi-Fi off while BLE owns it. The ROS router endpoint is compile-time (`ROVI_ZENOH_ROUTER`, default `tcp/192.168.4.2:7447`, [ADR-0012](../../../docs/adr/0012-ros-router-over-tcp.md)). `esp-radio` is built with `coex`; concurrent BLE and Wi-Fi traffic has not been characterized since M3 because [ADR-0004](../../../docs/adr/0004-single-owner-latest-setpoint.md) made it rare.

Todor's constraint: extend the current protocols and implementation minimally so existing tests and board evidence keep holding. Fallback mode must be byte-for-byte today's behaviour.

## Goals / Non-Goals

**Goals:** the boot/override state machine in the specs; one network stack per mode; a per-mode ROS endpoint; BLE preemption only in home mode; host tests for every transition before any board run.

**Non-Goals:** AP+STA; changes to setpoint, Status, Echo, lease, claim or arming rules; ROS router discovery; WPA3; retry from fallback without reboot.

## Decisions

### D1. A network supervisor owns the mode (decided by Todor: protocol steps 1–3)

A firmware task runs `attempt home → serve → (BLE override | link loss) → attempt home → … | fallback (terminal until reboot)`. The pure transition logic lives in `rovi-core` (testable on the host and in the simulator); the firmware task executes scan/join/DHCP/router steps with the budgets and reports outcomes to it. Alternative considered: deciding inside `wifi_gate` — rejected, it would mix radio switching with mode policy and is not host-testable.

### D2. Station and AP are exclusive; the stack is rebuilt per mode

Station mode uses `wifi::Interface::station()` with DHCPv4; fallback uses the existing AP interface and static address. A mode change stops the radio interface, tears down the stack-dependent tasks (UDP, ROS) and restarts them on the new stack. Alternative: AP+STA with channel sharing — rejected by Todor (AP only as fallback). Turning the station off for a BLE takeover must work without a reset (it happens with a BLE link up). If `esp-radio` 1.0.0-beta.1 cannot switch between station and AP at runtime, the home→fallback switch may use a software reset into a forced-fallback flag held in RTC memory, but only when no BLE connection exists; the extra `boot` event must be logged (builder's call).

### D3. BLE preemption is an arbiter rule gated on mode

`Device` gains a `mode` (Home/Fallback). In Home, `radio_allowed(Ble)` is always true and `on_ble_connect` with a Wi-Fi owner raises `StopReason::OwnerLost` for that owner, releases it and claims for BLE disarmed. No new stop reason or Status code, so clients and the wire format are unchanged. Alternative: a new `Preempted` reason — rejected, it changes the Status encoding on all three transports for little information gain; the log's mode/claim events already show what happened.

### D4. Any BLE disconnect in home mode returns to step 1 (decided by Todor)

This includes an accidental drop. Consequence: a ROS controller that is still streaming drives again within the join budget, with no operator step (ADR-0011 auto-arm). Recorded in the new ADR as an accepted risk; the mitigation is that the operator disarms and the controller should stop streaming when not wanted.

### D5. Home settings are build variables (decided by Todor: option 3)

`ROVI_HOME_SSID`, `ROVI_HOME_PASSWORD` and `ROVI_HOME_ROUTER` are read with `option_env!` like `ROVI_WIFI_PASSWORD`; a `const` check (or `build.rs`) fails the build when the set is partial or invalid. The fallback-mode router endpoint stays `ROVI_ZENOH_ROUTER` / `tcp/192.168.4.2:7447`. Changing networks means rebuilding and flashing over USB. Alternatives rejected: BLE provisioning (passphrase over unencrypted BLE, ADR-0008), BLE with pairing (more work on car and clients), a USB serial settings command with flash storage (new storage and protocol for a rare operation).

### D6. UDP stays available in home mode

The UDP task binds to whichever stack is active, so a UDP controller on the home network works with no extra code; BLE preempts it like ROS. Trust is the home network ([ADR-0008](../../../docs/adr/0008-local-link-security-baseline.md) superseded for station mode). This is the builder-side reading of "everyone on the private network may access the car".

### D7. Budgets

Scan ≤1500 ms (spec); join + DHCP + router ≤5000 ms after the network is found. Every home-mode attempt (boot, after a BLE disconnect, after station loss) uses the same budgets, starting when the attempt starts. Builds without the `ros` feature ignore home settings and always fall back (home mode requires a ROS router). Values are compile-time constants, logged per step so they can be tuned from board data. The router step reuses the existing zenoh session open with a deadline instead of its retry loop.

## Risks / Trade-offs

* [BLE advertising while ROS streams over Wi-Fi adds coexistence jitter] → board check of ROS round trip and Echo loss with advertising on, compared with the 2026-10-20 ROS soak baseline.
* [Accidental BLE drop hands control back to an auto-arming ROS stream] → accepted by Todor; documented in the ADR, README and the BLE client's quit message.
* [Anyone in BLE range can preempt the car in home mode] → unchanged ADR-0008 BLE risk; the home passphrase never goes over the air.
* [Runtime stack switch unsupported by the beta radio driver] → D2's reset fallback.
* [DHCP address of the router host changes] → the endpoint is built in by IP; give the router host a DHCP reservation or rebuild. Discovery is out of scope.
* [Passphrase in the firmware image] → the image is local and flashed over USB; it is not published. Like `ROVI_WIFI_PASSWORD`, the variables come from the shell environment (e.g. gopass), never from a committed file such as the tracked `.envrc`.

## Migration Plan

Without home settings the build behaves exactly as today, so existing images, clients and bench procedures need no change. Rollback is reflashing the previous image.

### D8. BLE release ends the connection

As today (`fw.rs` disconnects a BLE central that lost ownership), a BLE owner released by its claim or reclaim window is disconnected; in home mode that disconnect starts a home-mode attempt. So Wi-Fi never comes back while a BLE central stays connected.

## Open Questions

* Whether the ROS router step should also wait for the controller's liveliness tokens — settle from board data; does not change the specs.
