# Observational conformance on the car, 2026-11-01 (add-rovi-protocol 3.3)

Read-only run with motor power off: no ARM accepted, no setpoint sent (Todor, 2026-11-01: read-only checks run without a "go").

* Firmware: the protocol stack's top (`feat/protocol-differences-doc`, `a53d85d`), ROS + home settings, router `tcp/blackbox.local:7447`; ELF sha256 `35afa34cf2322616`. Home mode with the ROS router 4.4 s after boot ([car.log](car.log)).
* Suite: [`apps/simcar/scenarios/observe.py`](../../../apps/simcar/scenarios/observe.py) with `ROVI_OBSERVE=device`, car at 192.168.1.193:7777, Bluetooth via BlueZ/Bleak. The same rules run against the sim car in CI.

## Result: 5/5 pass ([observe.log](observe.log))

* INFO decodes under the crate and meets the single client minimum.
* Echo returns its 32-byte payload; a 33-byte Echo gets no reply.
* Four malformed frames (stray HELLO byte, bad magic, version 2, short header) get no reply and do not claim.
* HELLO claims disarmed and HELLO_ACK capabilities equal INFO's; a second client gets BUSY; its ARM with the owner's session gets no reply and changes nothing; BYE releases.
* Bluetooth: Capabilities, Status and Network reads equal UDP INFO; Echo round-trips.
* Serial: one BLE connection, one UDP claim and release, wheels 0 throughout, no arm events.

## Notes

* A first attempt attached `espflash monitor --no-reset` to the running car; the tool entered the bootloader (flash stub) and stopped the app, so all rules timed out. The board was then flashed with the top-of-stack build and captured with a normal monitor; that run is the one recorded.
* Home SSID redacted.
