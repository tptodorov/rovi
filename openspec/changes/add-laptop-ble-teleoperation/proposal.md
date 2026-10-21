# Proposal

## Why

Todor requested BLE support in the laptop keyboard client so it can control the
car without joining its Wi-Fi AP.

## What Changes

- Add mutually exclusive `--ble [MAC]` and existing `--target IPv4:port` modes.
  Bare `--ble` discovers one advertising `Rovi M8`; ambiguous matches fail.
- Reuse the existing keys, gentle limits, input expiry and loss tone. BLE starts
  and reconnects disarmed, requiring a new R and direction after recovery.
- Validate GATT service/capabilities and the capacity for a 72-byte setpoint;
  use native Status/Echo and connection generations instead of UDP framing.
- Keep bounded zero/STOP/disconnect cleanup and quit during discovery/retry.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `laptop-teleoperation`: selectable BLE transport, GATT validation, control
  confirmation, reconnect and disconnect cleanup.

## Impact

Python client, tests, usage and OpenSpec only. BLE uses Bleak already declared
in shell.nix. UDP needs no BLE package. No firmware, iPhone workspace, wireless
policy or system configuration changes. Stacked on PR #96's disarmed recovery.

## Acceptance

Todor explicitly authorized implementation on 2026-10-20: “yes, add BLE support
to the keyboard client”, after the same controls/reconnect/beep scope was
presented. Agent review preserves explicit arming and transport ownership;
there are no unresolved implementation decisions. Each physical motor test
still requires its separately announced go.
