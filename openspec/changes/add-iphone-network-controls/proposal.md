# iPhone network controls

**Status:** accepted for implementation, 2026-10-28. Todor said “do it” after the recommendation to add iPhone Home/Car network controls and network status (#109). No motor run is authorized. Real iPhone/Bluefy validation remains separate from host/browser evidence.

## Why

The car already accepts network mode commands over BLE, but the phone cannot display its network information or request a mode. Users currently need the laptop to do this.

## What Changes

- Show current/last-reported network mode, pending request, fallback reason and available home IPv4.
- Offer Home Wi-Fi and Car network actions while connected and disarmed. Confirm the command through fresh Status and Network reads, then disconnect Bluetooth so the request can take effect.
- Explain refusal, unavailable/old firmware and unconfirmed requests without claiming that the car has joined a network.
- Keep the existing arming, motion, cleanup and reconnect rules.

## Capabilities

### New Capabilities

- `iphone-network-controls`: network display and disarmed network requests in the existing Bluefy client.

### Modified Capabilities

None. Firmware commands and encodings are unchanged; the home-mode capability remains in its own active proposal.

## Impact

`apps/iphone` wire decoder, optional BLE characteristic, session procedure, UI and tests. No dependency, firmware, transport protocol or provisioning change. Closes #109 when implemented; no shared-client migration is required.
