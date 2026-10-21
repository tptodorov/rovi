# Verification map

| Contract | Host evidence | Required phone/physical evidence |
| --- | --- | --- |
| Default Buttons; permission only on tap; no automatic ARM | Browser and Session regressions | Bluefy mode/permission behavior |
| Three relative angles, wrap, dead zone, per-axis clamps | Physical unit tests | Phone signs, held twist, 60 s neutral drift; stand mixed axes |
| Independent sample/UI deadlines and no old input replay | Fake-clock input and Session tests, browser stalls | Phone stalls and observed manual recovery |
| Mode changes supersede ARM, clear/disarm/STOP | Fake-GATT race tests; real page switch while held | Phone and separately authorized stand stop |
| Lifecycle/reference/posture/invalid sample cleanup | Unit/Session/browser tests | Phone lock/app switch/orientation and stand interruption |
| Existing stream/Status/cleanup limits | Existing and new Session regressions | Actual sensor-plus-BLE gaps, STOP/readback; physical observation |

Pre-implementation sensor-only [phone result](../../../firmware/bench/2026-10-21-iphone-sensors/notes.md)
supports the selected interaction, not these product acceptance gates. No task 3
or sensor-driven rover result is claimed yet.
