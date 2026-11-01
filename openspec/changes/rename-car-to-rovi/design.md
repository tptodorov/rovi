# Design

## Context

See [proposal.md](proposal.md) for why. The firmware has the name in three places (AP SSID, GAP name, scan response). Clients match it exactly.

## Goals / Non-Goals

**Goals:** one advertised name, `Rovi`, on both radios; every current client finds it.

**Non-Goals:** changing frozen experiments; renaming the `rovi-m8` profile, the milestone or its paths; a shared name constant across the car and clients.

## Decisions

### D1. No fallback to the old name

Clients look only for `Rovi`. Accepting both would keep the old name in every client for one car and one transition.

### D2. Product bench scripts override the frozen client's name

`firmware/bench/ble_stop_checks.py` sets `ble_client.NAME`, which `find()` reads at call time, and `ble_lease_timing.py` shares that module. The frozen client keeps working against M8 firmware.

### D3. The `rovi-m8` profile keeps its name

It is a local identifier that bench scripts and procedures use, and it can keep working with one SSID change.

## Risks / Trade-offs

- **A host caches the old name** (BlueZ or iOS) → the car may not match until the host sees a fresh advertisement. Physical check 2.2 covers this.
- **Host tests cannot prove the radios** → the name on air needs the physical check ([ADR-0002](../../../docs/adr/0002-simulation-never-proves-hardware.md)).

## Migration

1. Flash the firmware and publish the phone page together. The page finds only a car running the new firmware, and the old page does not find it.
2. On the laptop, change the profile's SSID: `nmcli connection modify rovi-m8 802-11-wireless.ssid Rovi`. The profile is local, not declared in `~/mycfg`.

Rollback: reflash the earlier firmware and republish the earlier page.
