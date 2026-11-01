# Name the car Rovi on both radios

**Status: implemented and accepted.** On 2026-11-01 Todor asked: "i want to replace "Rovi M8" wifi and BLE connection everywhere, since it should be called just Rovi and so it should be advertised in the connections and protocols as such". BLE/Wi-Fi names, laptop discovery and publication passed on 2026-11-01 ([evidence](../../../../firmware/bench/2026-11-01-rovi-name/notes.md)); no new physical Bluefy check or AP association is claimed.

## Why

The product car still advertises the name of milestone M8, which it outgrew. The name is what people see when they pick the car on a phone or laptop, and nothing in the specification states it.

## What Changes

- The car advertises exactly `Rovi` as its BLE name and car AP SSID. **BREAKING** for clients that look for the old name.
- The laptop client, phone page, observation harness and product bench scripts look for `Rovi`.
- The old name stays in `experiments/`, whose M8 firmware and client are frozen. Product bench scripts override the frozen client's name instead of editing it.

No GATT, UDP or ROS byte changes. The AP password, address and channel are unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `car-control-api`: states the name the car advertises on both radios.
- `laptop-teleoperation`: BLE discovery selects `Rovi`.

## Impact

- `firmware/src/fw.rs`, `apps/teleop`, `apps/iphone/transport.js`, `apps/simcar/scenarios/observe.py`, `firmware/bench/`, current docs.
- Release: the firmware and phone page go together; see [design.md](design.md#migration).
- The laptop's `rovi-m8` NetworkManager profile needs its SSID changed.
