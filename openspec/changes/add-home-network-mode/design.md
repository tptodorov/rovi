# Design

## Context

See [proposal.md](proposal.md) for why. Today `firmware/src/fw.rs` builds one embassy-net stack on the AP interface with a static `192.168.4.1/24`, and a `wifi_gate` task switches the AP off while BLE owns the car. `rovi-core`'s `Device::radio_allowed` stops BLE advertising while a Wi-Fi source owns the car; `wifi_allowed` turns Wi-Fi off while BLE owns it. The ROS router endpoint is compile-time (`ROVI_ZENOH_ROUTER`, default `tcp/192.168.4.2:7447`, [ADR-0012](../../../docs/adr/0012-ros-router-over-tcp.md)). `esp-radio` is built with `coex`; concurrent BLE and Wi-Fi traffic has not been characterized since M3 because [ADR-0004](../../../docs/adr/0004-single-owner-latest-setpoint.md) made it rare.

Todor's constraint: extend the current protocols and implementation minimally so existing tests and board evidence keep holding. Fallback mode must be byte-for-byte today's behaviour.

## Goals / Non-Goals

**Goals:** the boot/override state machine in the specs; one network stack per mode; a per-mode ROS endpoint; BLE preemption only in home mode; host tests for every transition before any board run.

**Non-Goals:** AP+STA; changes to setpoint, Status, Echo, lease or claim-window rules; BLE preemption in fallback; ROS router discovery; WPA3; retry from fallback without reboot.

## Decisions

### D1. A network supervisor owns the mode (decided by Todor: protocol steps 1–3)

A firmware task runs `attempt home → serve → (BLE override | link loss) → attempt home → … | fallback (terminal until reboot)`. The pure transition logic lives in `rovi-core` (testable on the host and in the simulator); the firmware task executes scan/join/DHCP/router steps with the budgets and reports outcomes to it. Alternative considered: deciding inside `wifi_gate` — rejected, it would mix radio switching with mode policy and is not host-testable.

### D2. Station and AP are exclusive; one network stack per boot (builder's call, 2026-10-23)

A boot uses either the station interface with DHCPv4 (home attempt) or the AP interface with the static `192.168.4.1/24` (fallback), and the existing UDP and ROS tasks run unchanged on that one stack. Switching between them is a software reset with a one-word flag in persistent RTC memory: `SWITCH | reason` boots into fallback with that reason; power-on and every other reset attempt home. A BLE takeover in home mode does not reset: the station disconnects and later rejoins through a normal attempt. Considered: two live stacks (both interfaces exist in esp-radio 1.0.0-beta.1) with the UDP task on two sockets and the ROS adapter switching link managers — rejected for now: a second router link costs about 16 KB of RAM and would change the M8-validated UDP/ROS code paths, while mode switches are rare (a mode command, a failed attempt, a lost home network). Costs: an `ev=boot reset=Sw` line, a BLE watcher is dropped, and the reboot counts against the 2500 ms "AP up" budget, which board check 6.1 measures. AP+STA stays rejected by Todor.

### D3. Preemption is one arbiter rule: BLE > UDP > ROS

`Device` gains a `mode` (Home/Fallback/Attempting). A claim from a higher-priority source preempts a lower one: UDP over ROS in both modes (same radio, no radio change); BLE over UDP/ROS in home mode only (in fallback BLE is not advertising while Wi-Fi owns, as today). The old owner gets `StopReason::OwnerLost` and is released; the new owner starts disarmed. No new stop reason or Status code. Alternative: a new `Preempted` reason, rejected because it changes Status on all three transports; the log shows it.

### D4. Hand-back latch (decided by Todor: safe-by-default hand-back)

Preempting a ROS owner sets a latch in `Device`. When the preempting owner is released with Home pending, the latch clears and ROS auto-arms as today; otherwise the latch stays and the next ROS owner is claimed disarmed until it publishes `/rovi/arm` true (the existing STOP-latch path). This reverses the earlier "any BLE drop hands back immediately" for accidental drops only.

### D5. Home settings are build variables (decided by Todor: option 3)

`ROVI_HOME_SSID`, `ROVI_HOME_PASSWORD` and `ROVI_HOME_ROUTER` are read with `option_env!` like `ROVI_WIFI_PASSWORD`; a `const` check (or `build.rs`) fails the build when the set is partial or invalid. The fallback-mode router endpoint stays `ROVI_ZENOH_ROUTER` / `tcp/192.168.4.2:7447`. Changing networks means rebuilding and flashing over USB. Todor's home settings live in gopass entry `env/rovi/home` (`ROVI_HOME_SSID`, `ROVI_HOME_PASSWORD`, `ROVI_HOME_ROUTER`); load them per shell with `secenv env/rovi/home` before building. Alternatives rejected: BLE provisioning (passphrase over unencrypted BLE, ADR-0008), BLE with pairing (more work on car and clients), a USB serial settings command with flash storage (new storage and protocol for a rare operation).

### D6. Mode command and network information on every transport

Same rule everywhere (owner, disarmed, pending until release), parsed in `rovi-core`:

| | Mode command | Network information |
| --- | --- | --- |
| BLE | Control codes 3 (Home), 4 (Car network) | New `Network` characteristic `d47a0016-…`, read + notify |
| UDP | Kind 10, payload `1`/`2`, reply `STATUS` | Kind 11, any peer, like Echo |
| ROS | `/rovi/mode` `std_msgs/UInt8` from the owner node | `/rovi/network` `std_msgs/String` logfmt, ≥1 Hz and on change |

Network information is `mode=… pending=… fallback=… ip=…` (BLE/UDP: a fixed 8-byte binary form: mode, pending, reason, reserved, IPv4). UDP is available in home mode with no extra code because the UDP task binds to whichever stack is active; trust is the home network ([ADR-0008](../../../docs/adr/0008-local-link-security-baseline.md) superseded for station mode).

### D7. Budgets

Scan ≤1500 ms (spec); join + DHCP + router ≤5000 ms after the network is found. Every home-mode attempt (boot, after a BLE disconnect, after station loss) uses the same budgets, starting when the attempt starts. Builds without the `ros` feature ignore home settings and always fall back (home mode requires a ROS router). Values are compile-time constants, logged per step so they can be tuned from board data. The router step reuses the existing zenoh session open with a deadline instead of its retry loop.

### D14. BLE is quiet while a home attempt runs (board-measured, 2026-10-26)

With both BLE slots advertising, the Wi-Fi scan took 2.7 s instead of 160 ms (coexistence), past the scan budget. `Device::radio_allowed(Ble)` is false while attempting; BLE advertises again in home mode or fallback. The attempt lasts at most about 5.5 s, during which the car cannot be driven anyway.

### D10. BLE claims on the first command; two connection slots

`HostResources` grows to two connections. A connection is unclaimed until its first Setpoint or Control write, which calls the existing claim path (so preemption, BUSY and the claim window apply then). Reads, notifications and Echo never claim. Losing ownership still disconnects that central (D8); the other stays. Existing clients write ARM or a setpoint right after connecting, so they claim as before. Alternative: one slot with an idle timeout, rejected because a watcher would block the rescue controller.

### D11. One UDP observer query

UDP kind 11 `INFO` is answered to any peer, like Echo, with capabilities (the `HELLO_ACK` payload), Status and network information in one datagram. One query instead of three keeps the UDP surface small and lets a watcher poll everything it needs.

### D12. ROS release and takeover notice

`/rovi/release` (`std_msgs/Empty`, owner node only) calls the same release path as UDP BYE. On preemption the arbiter emits a final Status for the old owner before switching: firmware sends it as a UDP `STATUS` to the old endpoint or publishes `/rovi/status`; BLE, the highest rank, is never preempted.

### D13. One conformance suite for all transports

The shared rules (claim, arm, stop, lease, preemption, mode command, hand-back, observation, release, errors) are written once as transport-independent scenarios and run against the simulator through three adapters: UDP, ROS and a BLE GATT adapter. New rules are added once and proven on all three. Board acceptance stays separate (ADR-0002).

## Risks / Trade-offs

* [Two BLE connections plus Wi-Fi traffic raise coexistence load and RAM use] → measured in 6.3; watchers are read-only and notifications are small.
* [Clients that relied on "connect = claim" to see `claimed` at once] → the laptop and iPhone clients write first; their tests are rerun (task 5.1).

* [BLE advertising while ROS streams over Wi-Fi adds coexistence jitter] → board check of ROS round trip and Echo loss with advertising on, compared with the 2026-10-20 ROS soak baseline.
* [A manual takeover ends by accident while ROS still streams] → hand-back latch (D4): ROS needs `/rovi/arm` true. Stock tools without an arm publisher cannot resume after an accidental drop; a deliberate Home hand-back keeps them working.
* [Any home-network UDP client can take the car from a ROS controller] → accepted: the home network is trusted, and the preempted car stops disarmed.
* [Anyone in BLE range can preempt the car in home mode] → unchanged ADR-0008 BLE risk; the home passphrase never goes over the air.
* [Runtime stack switch unsupported by the beta radio driver] → D2's reset fallback.
* [DHCP address of the router host changes] → the endpoint is built in by IP; give the router host a DHCP reservation or rebuild. Discovery is out of scope.
* [Passphrase in the firmware image] → the image is local and flashed over USB; it is not published. Like `ROVI_WIFI_PASSWORD`, the variables come from the shell environment (e.g. gopass), never from a committed file such as the tracked `.envrc`.

## Migration Plan

Without home settings the build behaves exactly as today, so existing images, clients and bench procedures need no change. Rollback is reflashing the previous image.

### D8. BLE release ends the connection

As today (`fw.rs` disconnects a BLE central that lost ownership), a BLE owner released by its claim or reclaim window is disconnected; in home mode that disconnect starts a home-mode attempt unless Car network is pending. So Wi-Fi never comes back while a BLE central stays connected.

### D9. Leaving fallback

Fallback has no automatic retry (it would blink the AP off every scan). A Home command from any owner, applied at release, is the only way back without a reboot.

## Open Questions

* Whether the ROS router step should also wait for the controller's liveliness tokens — settle from board data; does not change the specs.
