# Design

Use the existing Network characteristic (`d47a0016-45b2-4d19-8db0-6bd87e9c0001`), BLE Control 3 (Home) and 4 (Car network). Keep the four required characteristics unchanged; Network is optional for older firmware. Read its initial value and subscribe to notifications when supported; read it again to confirm a pending command.

Decode the existing eight-byte layout strictly: mode, pending, fallback reason, reserved zero and four IPv4 octets. Unsupported/malformed network information disables network controls, without granting or revoking motion permission. The core Status checks remain authoritative for control safety.

Extend the existing single-operation session procedure with a network command. Reject invalid targets or armed/busy sessions locally. Submit zero, write the mode command with response, read fresh Status, then read fresh Network and require the matching pending request. Zero setpoints between GATT round trips retain the existing 200 ms stream guard without relaxing the control deadline. A refused command remains connected/disarmed; a failed or unconfirmed operation closes through existing cleanup. Successful confirmation also uses existing cleanup/disconnect, preserving pending mode through STOP. Never send ARM.

The network panel uses explicit action buttons rather than a persistent toggle: the mode applies only after Bluetooth ownership ends, and Home is also the deliberate hand-back command when already in home mode. Show last-reported information after disconnect, with a reconnect-to-check message. An all-zero home address is unavailable; it is normal while BLE owns the car and Wi-Fi is paused. Do not expose SSID/password fields.

Tests exercise decoding, optional/legacy characteristic discovery, notifications and listener cleanup, both command paths, refusal, stalled work, generation changes and armed guards. Chromium tests cover the rendered controls and absence of ARM/nonzero commands. Physical phone validation is a separate acceptance task; this change does not close the deferred range test in home mode.

Firmware rejects the acknowledged ATT write on refusal. The client reads fresh Status after either write outcome; only an acknowledged write plus error-free disarmed Status and matching pending Network can confirm success. A reported refusal stays connected; an unexplained failed write closes as unconfirmed.
