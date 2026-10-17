# Milestones

## M4: four motors on the M8 firmware

**Status:** **complete (2026-10-17).** Step 1 (one motor) passed 2026-10-14; step 2 (four motors on the soldered carrier) passed 2026-10-17. Open, not gating: motor capacitors C3-C6, the motor power bank's auto-sleep.
**Setup:** M1's wiring plus motors and the 5 V power bank ([WIRING.md](WIRING.md)); wheels off the ground.

### Goal

Drive the four motors from the M8 firmware and show that every safety stop stops them. Record the numbers M6 and M5 need: wheel mapping and inversion (O8), the stop bound (O4), whether starts and reversals reset the board (O7), and the duty and current limits (O1).

Simplified on 2026-10-14 ([ADR-0014](../../docs/adr/0014-m4-minimal-wiring-software-fail-safe.md)): no current monitor, hardware dead-man, bench supply or logic analyzer. The earlier wiring draft was invalid and is retired.

### Steps

1. **One driver, one motor** (M1's wiring): `STBY` follows arming, the motor follows speed and direction, and STOP, lease expiry, disconnect, reset and power loss stop it ([TESTPLAN](TESTPLAN.md) S1-S5).
2. **Four motors, two drivers:** mapping and inversion per wheel, all four at increasing speed with no brownout reset and no power-bank cut-out, the stops with all four turning, a thermal run.

### Acceptance

Real hardware only.

- [x] `STBY` is low through boot, reset and while disarmed; high only while armed. *(Step 1: shown by S3 on one motor; S1's meter check skipped.)*
- [x] Each motor follows speed and direction; mapping and inversion recorded for all four wheels. *(Step 2, 2026-10-17: every channel drives its intended wheel, all forward; directions set at the connectors, no firmware inversion. [notes](bench/2026-10-17-step2/notes.md))*
- [x] STOP, lease expiry, controller disconnect, reset and ESP32 power loss each stop every wheel, and nothing restarts until a client arms again. *(Step 1 with one motor; step 2, 2026-10-17, with all four turning at 70 %.)*
- [x] Four motors starting and reversing together do not reset the ESP32 or cut out the power bank. *(Step 2, 2026-10-17: 10 reversals at ±70 %, no reset, bank stayed on; the bank must be woken before a session.)*
- [x] The radio keeps working while the motors run. *(Step 1: 5 min at 40 %; step 2: 5 min at 70 % with four motors; no reboot, no BLE drop.)*
- [x] Driver and motor stay at most warm in a 5 min run; duty cap and current recorded for M5. *(Step 2: 5 min at 70 %, at most warm. Duty range 40-100 %, start threshold 35 %, 150 mA per motor running.)*

### Results

#### 2026-10-16/17 step 2: four motors on the carrier — pass

Notes and logs: [2026-10-16](bench/2026-10-16-step2/notes.md), [2026-10-17](bench/2026-10-17-step2/notes.md), [setup photo](bench/2026-10-17-step2/setup.jpg).

* **Mapping and direction:** every channel drives its intended wheel, all forward (directions set at the connectors, no firmware inversion); strafe and rotate patterns as expected.
* **Start duty:** the mounted motors need about **35 %** with all four connected (a loose motor started at 9 %). The motor layer now maps any command outside a 2 % deadband to **40-100 %** duty (PR #42); the gentlest command starts every motor (O6, M4 part).
* **Power:** the motor power bank **auto-sleeps at 2.7 V** (below the TB6612's 4.5 V VM minimum) until its button is pressed; awake, VM is 4.7 V under load. Running current about 150 mA per motor.
* **Reversal:** 10 changes at ±70 %, no ESP32 reset, bank stayed on (O7: no slew limit needed at these duties).
* **Stops with four turning:** STOP, lease expiry (1 s after the last setpoint), controller loss, RST and ESP32 power loss each stopped all four; nothing restarted until a client armed again (O4).
* **Thermal:** 5 min at 70 %, drivers and motors at most warm; radio fine throughout.
* **Fault found and fixed:** the FR motor would not start on its own; a plug swap showed it followed the motor, and Todor's rework fixed it.

#### 2026-10-14 step 1: one driver, one motor — pass

M8 firmware with motor outputs, M1's wiring, power bank on VM, BLE client ([notes and logs](bench/2026-10-14-step1/notes.md)). The motor followed speed (10, 20, 40 % duty) and direction. STOP, lease expiry (1.0 s after the last setpoint), a killed controller (stopped by the lease 1 s later), the RST button and unplugging the ESP32 each stopped it, and it stayed off until a client armed again. A 5 min run at 40 % had no reboot or BLE drop, and the driver stayed cold. 
**Start duty, no load** ([notes](bench/2026-10-14-start-duty/notes.md)): from standstill 8 % duty twitches and **9 % turns**; while running it keeps turning down to about 8 % and **stops at about 7 %**. Input to O6 (dead zone); under load on the floor (M5) both will be higher.

#### Motor current (measured by Todor)

* 2026-10-03: one motor on the 5 V power bank drew **120 mA** (conditions not recorded).
* 2026-10-14: **150 mA per motor running** (Todor's measurement). Four motors running: about 0.6 A, within the TB6612FNG's 1 A per channel and a power bank port's rating. Start and stall currents are not measured; the power bank's protection and the TB6612's thermal shutdown bound them.
