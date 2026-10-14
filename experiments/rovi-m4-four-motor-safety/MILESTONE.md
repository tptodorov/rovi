# Milestones

## M4: four motors on the M8 firmware

**Status:** step 1 ready for the bench (2026-10-14); nothing run on hardware.
**Setup:** M1's wiring plus motors and the 5 V power bank ([WIRING.md](WIRING.md)); wheels off the ground.

### Goal

Drive the four motors from the M8 firmware and show that every safety stop stops them. Record the numbers M6 and M5 need: wheel mapping and inversion (O8), the stop bound (O4), whether starts and reversals reset the board (O7), and the duty and current limits (O1).

Simplified on 2026-10-14 ([ADR-0014](../../docs/adr/0014-m4-minimal-wiring-software-fail-safe.md)): no current monitor, hardware dead-man, bench supply or logic analyzer. The earlier wiring draft was invalid and is retired.

### Steps

1. **One driver, one motor** (M1's wiring): `STBY` follows arming, the motor follows speed and direction, and STOP, lease expiry, disconnect, reset and power loss stop it ([TESTPLAN](TESTPLAN.md) S1-S5).
2. **Four motors, two drivers:** mapping and inversion per wheel, all four at increasing speed with no brownout reset and no power-bank cut-out, the stops with all four turning, a thermal run.

### Acceptance

Real hardware only.

- [ ] `STBY` is low through boot, reset and while disarmed; high only while armed. *(S1)*
- [ ] Each motor follows speed and direction; mapping and inversion recorded for all four wheels. *(S2, step 2)*
- [ ] STOP, lease expiry, controller disconnect, reset and ESP32 power loss each stop every wheel, and nothing restarts until a client arms again. *(S3, step 2)*
- [ ] Four motors starting and reversing together do not reset the ESP32 or cut out the power bank. *(step 2)*
- [ ] The radio keeps working while the motors run. *(S4)*
- [ ] Driver and motor stay at most warm in a 5 min run; duty cap and current recorded for M5. *(S5, step 2)*

### Results

#### Motor current (measured by Todor)

* 2026-10-03: one motor on the 5 V power bank drew **120 mA** (conditions not recorded).
* 2026-10-14: **150 mA per motor running** (Todor's measurement). Four motors running: about 0.6 A, within the TB6612FNG's 1 A per channel and a power bank port's rating. Start and stall currents are not measured; the power bank's protection and the TB6612's thermal shutdown bound them.
