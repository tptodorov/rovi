# Milestones

## M4: four-motor power and safety qualification

**Status:** proposed — preliminary motor observation recorded; full M4 setup not yet implemented or qualified
**Setup count:** one, integrated four-motor chassis bench setup

### Goal

Establish safe electrical limits for the received motors, TB6612FNG boards, battery, wiring, and regulators, and verify that every motor output remains disabled during reset, startup, controller loss, and watchdog expiry.

### Risks covered

* Unknown motor voltage/current/speed and driver-board current/thermal capability.
* Battery, protection, wiring, connector, and regulator capacity during motor starts and combined loads.
* Logic-rail sag, brownout, and recovery.
* Motor switching noise and current-return paths affecting the ESP32 or control links.
* GPIO boot/reset state and all-driver standby fail-safe.

### Test plan

1. Identify motor labels, driver-board markings, battery/protection ratings, and connector/wire limits. Compare known ratings to the TB6612FNG datasheet and the received-board reference before applying load.
2. A preliminary 120 mA reading at 5 V exists, but its load and method are unknown. With wheels clear and a current-limited supply, separately record each identified motor's no-load voltage, current, and speed if measurable. Use only brief, current-limited stall measurements if the setup can enforce a safe limit; otherwise mark stall current unknown and do not claim the driver is qualified for it.
3. Drive each channel independently, then ramp two and four motors together. Record supply voltage/current, logic-rail minimum, ESP32 reset/brownout events, driver temperature, and motor temperature over the agreed duty/load range.
4. Exercise starts, stops, and direction reversals. Inspect ground returns, decoupling, and suppression; observe the logic rail and ESP32 operation while the motor wiring is switching.
5. Verify every STBY input has a hardware-safe inactive state while the ESP32 is unpowered, held in reset, booting, and recovering from reset. Confirm no motor starts before firmware explicitly enables the drivers.
6. With the integrated motor setup, trigger the configurable watchdog and controller disconnect. Confirm the timeout event occurs and all STBY signals go inactive within the specified bound, including during a multi-motor load.
7. Repeat representative loaded motor operation while sending direct Wi-Fi commands. Record Wi-Fi loss or controller resets associated with motor switching. BLE bring-up and link compatibility are out of scope.

### Acceptance

Real hardware only. Start with unloaded wheels and current-limited power; increase load only after recorded limits support it.

- [ ] Motor, driver, battery, wiring, connector, and regulator data are identified or explicitly marked unknown.
- [ ] Measured simultaneous current and temperatures stay within documented component limits with agreed margin.
- [ ] Motor starts/reversals do not reset the ESP32 or pull the logic rail below its operating limit.
- [ ] All motor outputs remain disabled through power-up, reset, and firmware recovery until explicitly enabled.
- [ ] Watchdog/controller loss disables every driver's standby signal within the product bound.
- [ ] Safe operating voltage, duty, current, and thermal limits are recorded for the next motion milestone.

### Results

Add date, firmware revision, wiring, instruments, supply limits, operating conditions, measurements, and pass/fail observations after bench testing.

#### 2026-10-03 preliminary motor-current observation

* One motor connected to a 5 V power bank drew **120 mA**, as reported by Todor.
* The power bank's stated capacity is **6500 mAh**.
* Motor identity, whether it was unloaded, measurement instrument/method, and whether 120 mA was steady-state or startup current were not recorded. Treat this as an initial observation, not a rated or stall-current measurement. See the [chassis reference](../../docs/reference/mecanum-car/README.md#preliminary-measurement).
* Arithmetic for this operating point is **0.60 W** (`5 V × 0.12 A`). Four motors at the same assumed draw would be **0.48 A / 2.4 W**; this is not a measured combined load. The reference documents the conditional energy/runtime arithmetic and its limitations.

Repeat the observation with the motor identified and mechanically unloaded, record meter placement/method and startup versus steady current separately, then measure representative loaded operation. Confirm the power bank's rated output capacity and whether it is the intended car supply before using capacity figures in a runtime estimate.
