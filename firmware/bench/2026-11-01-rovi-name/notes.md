# Rovi connection-name check, 2026-11-01

Firmware source: `fa117dd4052da51012e4870da46dc5a206814a46`, product firmware with `ros` and existing home settings. ELF SHA-256: `0d7ff38b7774e067af5853eb4bf5696c04e4595a19938d8e13db990b56c4ceab`. Vector SHA-256: `4bf34426ab0a946a5cd2f5244ca354ebc1e6bbdb790e83368cb17fb5655d31b1`.

No arming or motion commands. Flash and capture used one `espflash flash --monitor` process on USB; the capture was stopped after restoration to Home. Home SSID redacted.

## Results

- BLE scan and GAP Device Name both report exactly `Rovi` ([readback](ble-name.log)).
- All five observational rules pass against the real car at `192.168.1.193:7777`: BLE/UDP readback and Echo agree, malformed frames are ignored, claim/BUSY/not-owner/release works ([results](observe.log), [suite](../../../apps/simcar/scenarios/observe.py)). Its wrong-peer ARM is rejected; no accepted ARM or setpoint is sent.
- A disarmed BLE mode command selects Car network. The current laptop discovery code finds `Rovi`, its GAP name is `Rovi`, and Network reports fallback/requested. Wi-Fi scan finds SSID `Rovi` and no `Rovi M8` ([results](names.log), [reproducible script](names.py)).
- The script restores Home: mode home, pending none, IP `192.168.1.193`.
- Serial: six wheel-state samples, all zero; no arm events ([capture](car.log)).
- On `blade`, the existing NetworkManager profile `rovi-m8` now targets SSID `Rovi`, retaining its credential and profile identifier. It was inactive throughout; the laptop's home Wi-Fi connection was unchanged. No matching declarative profile exists in `~/mycfg`.

## Phone release

The new picker requests exactly `Rovi`; the 46 built-page Chromium checks and 69 Node tests passed. Build [`8e84bdaac9c2`](https://tptodorov.github.io/rovi/iphone/?v=8e84bdaac9c2) is published. The live version, picker asset (`Rovi`, no legacy name) and `application/wasm` response are verified ([publication record](published.json)). This build is the rebased equivalent of the flashed source; only the spec-reader content changed on rebase. This run does not claim a fresh physical Bluefy check or a Wi-Fi association to the renamed AP; it verifies the actual radio names and laptop BLE discovery.

## Initial attempt

The first BLE scan ran before flashing and booting finished and timed out. After the capture showed Home ready, the same name/GAP readback passed. No firmware change was required.
