---
status: accepted
---

# Local-link security baseline: WPA2 AP only, unpaired BLE, no station mode

Rovi's control links are local and short-range. We decided (2026-10-07) how much access control to build for M8 and the bench milestones that follow:

* **UDP: the WPA2 car AP is the only access control.** The AP allows one station, so only the holder of the passphrase can reach port 7777. There is no per-session token or message signing. The session id and sequence in `RoviHeader` reject replays and stray packets, and the header's version byte leaves room to add a MAC later.
* **BLE: unpaired and unencrypted, as in M2 and M3.** The risk is accepted because the connection (the claim) happens once at the start of a session and only at close range. There is no LE Secure Connections bonding.
* **No station mode.** The car runs its own AP only. `rmw_zenohd` runs on a laptop on that AP, so the car doesn't need to join an existing network.
* **M3's open items move to M8.** Range, phone onboarding and dedicated-radio latency run against M8's UDP firmware, not M3's superseded TCP path ([ADR-0004](0004-single-owner-latest-setpoint.md)).

## Considered options

* **A per-session token or MAVLink-style signing over UDP.** Rejected for now. It needs key provisioning and a counter, and the AP's single-station limit already restricts who can send.
* **BLE LE Secure Connections with bonding and encryption required on the Setpoint and Control characteristics.** Rejected for now. Just Works gives no MITM protection anyway, and the use is a single close-range connect.
* **Station mode.** Rejected for M8: it adds infrastructure dependence and no capability the laptop-on-AP setup lacks.

## Consequences

* A stranger in BLE range can connect first and hold ownership, or observe and inject setpoints on an open BLE session. Arming, the 1 s lease and the claim window bound what that can do, but they don't prevent it.
* Revisit this decision if:
  * motors run for real use with people or property nearby (M4/M5 onward);
  * the car is used outside a controlled space;
  * a product client such as the iPhone app exists;
  * ROS 2 on an existing lab network becomes a goal.
