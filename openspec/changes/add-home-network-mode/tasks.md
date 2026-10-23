# Tasks

Behaviour decided by Todor on 2026-10-22. Nothing here authorizes implementation, a flash or a motor run.

## 1. Acceptance

- [x] 1.1 Todor accepts this proposal and [ADR-0016](../../../docs/adr/0016-home-network-mode-with-ble-override.md) for implementation (ADR status → accepted). Record the date and wording here; strict `openspec validate` and `spec:reader:check` pass. **Done 2026-10-22:** Todor: “merge them and continue with next steps” on PR #107 after reviewing the aligned proposal; strict validation and reader check pass. No flash or motor run is authorized by this.

## 2. Core state machine (host)

- [x] 2.1 Add the network mode and supervisor transitions to `rovi-core` test-first: no settings → fallback; scan/join/DHCP/router failures → fallback; success → home; BLE disconnect in home → attempt; station loss → attempt; fallback terminal until reboot. Verify with unit tests covering every spec scenario in `network-mode`. **Done:** `rovi-core::network`, 24 unit tests.
- [x] 2.2 Add BLE > UDP > ROS preemption and the hand-back latch to the arbiter/`Device` test-first: UDP over ROS in both modes, BLE over Wi-Fi in home mode only, owner-lost stop, disarmed new owner, latch set/cleared per D4. Existing `rovi-core` tests pass; the ones whose behaviour changes are updated with a note. **Done:** 11 Device tests; two "ROS blocks UDP" tests now assert takeover; 247 workspace tests pass.
- [x] 2.3 Add mode and step events to the catalog (`events.rs`, `carlog.py`, `docs/LOGGING.md`) with tests; `test_carlog.py` passes. **Done:** `net`, `net_step`, `mode_cmd`, `preempted`; `test_carlog` catalog check passes.
- [x] 2.4 Add the mode command (BLE codes 3/4, UDP kind 10, ROS `/rovi/mode`) and network information (BLE characteristic, UDP kind 11, ROS `/rovi/network`) codecs test-first, with every error case in *Mode command errors*. **Done:** BLE codes 3/4, UDP `Mode` = 10, `on_ros_mode`, `Info` 8-byte/logfmt encodings, `take_mode_log`; 9 new tests.
- [x] 2.5 Add BLE claim-on-first-command (two connections, reads/Echo never claim), UDP `INFO` for any peer, ROS `/rovi/release` and the takeover-notice Status to `rovi-core` test-first, covering every scenario of those requirements. **Done:** `on_ble_attach` (claim on first write, two live slots, released owners cannot reclaim), UDP `Info` = 11, `on_ros_release`, `Notice` (UDP frame / ROS status); `on_ble_connect` keeps claim-at-connect until the firmware switches in 4.1. 10 new tests.

## 3. Simulator

- [x] 3.1 Model the mode in the sim car (network steps injected by the scenario) and add scenarios: deliberate and accidental hand-back, UDP takeover from ROS, mode commands from each transport, router restart in home mode, link loss → fallback, Home from fallback. All existing sim scenarios pass. **Done:** `net.rs` (stdin-scripted steps), `vble.rs` (virtual BLE), ROS `rovi/mode`, `rovi/release`, `rovi/network` and takeover notice; `network_scenarios.py` (14), `ros-network.sh` (UDP/BLE takeover from ROS, deliberate and accidental hand-back, ROS mode + release), `ros-reconnect.sh` in home mode; existing UDP (14) and product ROS scripts pass. All in CI.
- [x] 3.2 Build the transport-independent conformance suite (D13) with UDP, ROS and BLE GATT adapters against the simulator; port the shared existing scenarios into it and add every new car-control-api and network-mode scenario. CI runs it on every PR. **Done:** `conformance.py`/`conformance.sh`: 8 shared rules (arm rule, STOP latch, lease, out-of-limits, mode refused/applied, invalid mode, observation, release) × UDP, virtual BLE and ROS = 24 cases against one sim car in home mode, stable over 3 runs; in CI. Transport-specific checks (UDP sessions/sequences, ROS graph identity, BLE slots, takeover and hand-back) stay in `udp_scenarios.py`, `network_scenarios.py` and the ROS scripts rather than being duplicated here.

## 4. Firmware

- [ ] 4.1 Build-time home settings with build-failure checks for partial/invalid sets (verified by building each case); station interface with DHCPv4, per-mode router endpoint, per-mode stack rebuild (or D2's reset fallback), budgets from D7. Firmware builds with and without `ros`; CI green.
- [ ] 4.2 Check the passphrase appears in no log line, characteristic, Status or ROS topic (host test over the event catalog plus a board log grep in 6.1).

## 5. Client and docs

- [ ] 5.1 Laptop teleop: `--mode home|car`, network-information display, takeover notice shown as "overridden", BLE claim on first write; rerun the iPhone client tests against the new claim timing; fake-peer/GATT tests for success, refusal (armed, invalid) and display.
- [ ] 5.2 Write `docs/reference/protocol.md`: one row per operation, one column per transport (BLE, UDP, ROS), linked from the specs and client READMEs. Mark ADR-0004 and ADR-0008 as partly superseded by ADR-0016; README build command with the home variables, PLAN/GLOSSARY updates; reader regenerated.

## 6. Board acceptance (each step separately announced; motor steps need Todor's go)

- [ ] 6.1 Bare board, no motors: boot with no settings, out of range, wrong passphrase, no router, and success; log shows each step's duration within budget.
- [ ] 6.2 Stand: ROS node on the home network drives; UDP and BLE takeovers stop and need explicit ARM; deliberate hand-back resumes ROS; accidental BLE drop needs `/rovi/arm` true; mode commands from BLE, UDP and ROS switch modes both ways; Todor observes wheels.
- [ ] 6.3 Coexistence: 15-minute ROS soak in home mode with BLE advertising, compared with the 2026-10-20 ROS soak baseline; link loss by walking out of range falls back with the AP up.

## Workflow follow-up

- Archive after acceptance; close Q21 in `PLAN.md`.
