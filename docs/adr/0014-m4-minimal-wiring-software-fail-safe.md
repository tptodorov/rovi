---
status: accepted
---

# M4 uses minimal wiring and a software fail-safe; motor control lives in the M8 firmware

The M4 draft (2026-10-11) planned an INA226 current monitor, a 74HC123 hardware dead-man on `STBY`, a bench supply with fuse and kill switch, and a logic analyzer, driven by its own serial-command firmware. The wiring draft turned out to be invalid, and a measured running current of 150 mA per motor removed most of the unknowns those parts were for. We decided (2026-10-14):

* **Wiring:** the ESP32-S3, TB6612FNG drivers, the motors and the 5 V power bank, on M1's bench-proven pins. Nothing else.
* **Fail-safe in software:** `STBY` is driven from the device state (high only while armed), the 10 ms control tick feeds the RTC watchdog (1 s, resets the chip), and the TB6612's internal pull-down holds `STBY` low during reset and while the ESP32 is unpowered. On the bench, unplugging the power bank is the kill switch, and its own protection is the current limit.
* **Firmware:** motor outputs are added to the M8 firmware (`motor::output` in the core). The separate M4 firmware and its core are retired; M4 is driven by M8's BLE, UDP and ROS 2 clients.

## Considered options

* **Keep the 74HC123 dead-man.** Rejected for now: it covers a hang that leaves `STBY` high for less than the watchdog's 1 s, which is acceptable with the wheels off the ground and a hand on the power cable.
* **Keep a separate M4 serial firmware.** Rejected: it duplicates the ownership, lease and stop logic M8 already tests, and would be thrown away.

## Consequences

* A firmware hang can leave the motors running for up to 1 s before the watchdog resets the chip.
* Stop timing is checked by the car log and by watching the wheel, not measured on the pins; start and stall currents are bounded by the power bank, not measured.
* Revisit before the car drives near people or property (M5 onward): a hardware dead-man or a supply cut-off, and measured stall currents.
