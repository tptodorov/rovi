# Protocol crate on the car: read-only BLE and UDP check, 2026-11-01

Todor connected the car over USB and said "go" to the announced read-only check (motor power off, no ARM, no setpoints).

* Firmware: `feat/protocol-car-variants` (#128) at `c992e9d`, ROS + home settings, router `tcp/blackbox.local:7447`; ELF sha256 `6ba65c972a09a1ac`.
* Boot to Home mode with the ROS router in 2.2 s ([car.log](car.log)).

## Result: pass ([check.log](check.log), script [check.py](check.py))

* **GATT UUIDs (I7):** the service `d47a0010-…` and all six characteristics `d47a0011`–`d47a0016` are present after connecting. The `u128` constants from `rovi-protocol::ble::id` therefore produce the same GATT table as the former string literals.
* **Reads:** Capabilities, Status and Network decode with the crate's rules (version 1, consistent Status, zero reserved byte).
* **UDP INFO** from a watcher returns the same Capabilities, Status (open) and network information (home, 192.168.1.193) as BLE.
* **Malformed HELLO** (one stray payload byte) gets no reply and does not claim; the next INFO still reports open.
* **Serial:** no claim or arm events; one BLE connection up/down.

## Notes

* The advertisement carries only flags and the name "Rovi M8" (unchanged from `main`). The first script run filtered on the service UUID in the advertisement and found nothing; it was corrected to find the car by name and check UUIDs in the GATT table.
* The script used its own minimal decoders (no Python binding yet; task 3.1).
* Home SSID redacted in logs.
