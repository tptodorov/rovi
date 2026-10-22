# Tasks

Behaviour decided by Todor on 2026-10-22. Nothing here authorizes implementation, a flash or a motor run.

## 1. Acceptance

- [ ] 1.1 Todor accepts this proposal and [ADR-0016](../../../docs/adr/0016-home-network-mode-with-ble-override.md) for implementation (ADR status → accepted). Record the date and wording here; strict `openspec validate` and `spec:reader:check` pass.

## 2. Core state machine (host)

- [ ] 2.1 Add the network mode and supervisor transitions to `rovi-core` test-first: no settings → fallback; scan/join/DHCP/router failures → fallback; success → home; BLE disconnect in home → attempt; station loss → attempt; fallback terminal until reboot. Verify with unit tests covering every spec scenario in `network-mode`.
- [ ] 2.2 Add home-mode BLE preemption to the arbiter/`Device` test-first: advertising allowed with a Wi-Fi owner, owner-lost stop then disarmed BLE owner, explicit ARM required, no preemption in fallback. Existing `rovi-core` tests pass unchanged.
- [ ] 2.3 Add mode and step events to the catalog (`events.rs`, `carlog.py`, `docs/LOGGING.md`) with tests; `test_carlog.py` passes.

## 3. Simulator

- [ ] 3.1 Model the mode in the sim car (network steps injected by the scenario) and add scenarios: hand-back to a streaming ROS node, accidental BLE loss, router restart in home mode, link loss → fallback. All existing sim scenarios pass unchanged.

## 4. Firmware

- [ ] 4.1 Build-time home settings with build-failure checks for partial/invalid sets (verified by building each case); station interface with DHCPv4, per-mode router endpoint, per-mode stack rebuild (or D2's reset fallback), budgets from D7. Firmware builds with and without `ros`; CI green.
- [ ] 4.2 Check the passphrase appears in no log line, characteristic, Status or ROS topic (host test over the event catalog plus a board log grep in 6.1).

## 5. Docs

- [ ] 5.1 Mark ADR-0004 and ADR-0008 as partly superseded by ADR-0016; README build command with the home variables, PLAN/GLOSSARY updates; reader regenerated.

## 6. Board acceptance (each step separately announced; motor steps need Todor's go)

- [ ] 6.1 Bare board, no motors: boot with no settings, out of range, wrong passphrase, no router, and success; log shows each step's duration within budget.
- [ ] 6.2 Stand: ROS node on the home network drives; BLE takeover stops and needs explicit ARM; deliberate and accidental BLE drop hand back; Todor observes wheels.
- [ ] 6.3 Coexistence: 15-minute ROS soak in home mode with BLE advertising, compared with the 2026-10-20 ROS soak baseline; link loss by walking out of range falls back with the AP up.

## Workflow follow-up

- Archive after acceptance; close Q21 in `PLAN.md`.
