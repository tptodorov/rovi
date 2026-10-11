# M4 bench test plan

Ordered from the least to the most dangerous. Do not start a test until the one before it passed. **Nothing here has been run.** Wiring: [WIRING.md](WIRING.md); parts: [BOM.md](BOM.md); firmware and commands: [README.md](README.md). Every number is a *placeholder* until the test that measures it has run; the measured value then replaces it in `Config::BENCH`, in this file and in the M6 spec.

## Safety rules (every powered test)

* **Todor is present and at the kill switch (SW1) or PSU output button** for every test with VM connected. Nothing runs unattended. The agent never flashes or runs hardware.
* **Wheels never touch the floor in M4.** Any test with a motor connected has the chassis on blocks, wheels free; floor driving is M5.
* **Set the PSU current limit before connecting VM**, at the value the test states. Raise it only when the previous test's record supports it, one step at a time.
* VM off (SW1 open) before changing any wiring. One change between runs.
* Hands, hair, cables clear of the wheels. Safety glasses.
* **Stop at once** on smell, smoke, a part too hot to touch, a reset you did not command, a current at the PSU limit you did not expect, or any wheel moving in standby. Open SW1 first, then note what happened.
* Logic analyzer probes only on 3.3 V logic nets (WIRING probe list), never on motor outputs or VM.

## Evidence

One directory per run: `experiments/rovi-m4-four-motor-safety/bench/<YYYY-MM-DD>-t<N>-<name>/` containing:

* `car.log`: the serial log, captured by `tools/m4ctl.py` (format: [LOGGING.md](../../docs/LOGGING.md)).
* `*.sr`: logic-analyzer captures (`sigrok-cli ... -o`), opened with PulseView.
* `notes.md`: date, firmware git revision and any `Config::BENCH` edits, PSU voltage and limit, fuse, instruments, readings that are not in the log, pass/fail.
* Photos of the setup. Camera frames stay out of git (`.gitignore`); keep only a contact sheet.

The summary of each test goes into [MILESTONE.md](MILESTONE.md#results).

## Commands used below

```sh
cd experiments/rovi-m4-four-motor-safety
B=bench/2026-MM-DD-t5-one-motor; mkdir -p $B
tools/m4ctl.py /dev/ttyACM0 $B/car.log "en" "w 0.3 0 0 0 for 3000" "wait 500"   # stream, then let the watchdog fire
sigrok-cli -d fx2lafw -c samplerate=4m --time 5000 -t D2=f -w -o $B/stby.sr      # wait for STBY to fall
```

The serial port name and whether opening it resets the board are unknown; a reset shows as an extra `ev=boot` and is harmless (boot is standby).

## Tests

### T0: paper and continuity (nothing powered)

* **Setup:** everything wired per WIRING.md, PSU off, USB unplugged.
* **Procedure:** read and photograph motor, driver, INA226 and PSU markings; read the INA226 shunt marking; check every WIRING row with the multimeter (continuity); check no short VM-GND, 3V3-GND, VM-3V3.
* **Measure:** resistance VM-GND, 3V3-GND; shunt value.
* **Stop if:** any short, any unknown marking on a part that carries VM.
* **Pass:** wiring equals the netlist; no shorts; shunt value recorded and set in `SHUNT_MILLIOHM`.
* **Evidence:** `notes.md`, photos.

### T1: board only, no VM (drivers' logic powered, no motor supply)

* **Setup:** USB to the COM port, SW1 open, PSU off, motors unplugged. Logic analyzer capture A.
* **Procedure:** start a capture, then plug USB (power-up); press RST three times; flash the firmware (Todor). Then `en`, stream `w 0.3 0.3 0.3 0.3` for 2 s, stop streaming; `en`, `stby`; `en`, `w nan 0 0 0`. Repeat with capture B (all four PWM, D2 directions).
* **Measure:** `STBY` level through power-up, reset, flashing and boot; time from the last `w` line to `STBY` low (core watchdog, placeholder 200 ms); PWM frequency and duty per channel (0.3 × 400 ‰ = 12 %).
* **Stop if:** `STBY` high at any moment before `en`.
* **Pass:** `STBY` low through every power-up, reset and flash; high only after `en`; watchdog, `stby` and NaN each drop it; log shows `ev=boot`, `ev=motor state=Standby reason=Boot`, then `Watchdog`, `Disabled`, `NonFinite`; PWM as commanded on the right pins.
* **Evidence:** `car.log`, `powerup.sr`, `reset.sr`, `watchdog.sr`, `pwm.sr`.

### T2: hardware dead-man, no VM

* **Setup:** as T1, capture A, trigger on `STBY` falling.
* **Procedure:** `en`, stream `w 0 0 0 0`, then `hang high`; after the reboot repeat with `hang low`. Unplug USB while enabled (ESP unpowered). Hold RST while enabled.
* **Measure:** `tw` = last `HEARTBEAT` rising edge → `STBY` low; time to `ESP_RST` low (RTC watchdog, placeholder 1 s); reset cause in the next `ev=boot`.
* **Stop if:** `STBY` stays high after the heartbeat stops.
* **Pass:** `STBY` low within `tw` in both hang cases, with no firmware action; `tw` longer than 2 ticks plus margin (> 30 ms) and recorded; RTC watchdog resets the chip (`reset=SysRtcWdt` or similar); `STBY` low when unpowered and in reset. Tune R4/C7 here if `tw` is far from the target, and re-run.
* **Evidence:** `hang-high.sr`, `hang-low.sr`, `unplug.sr`, `car.log`.

### T3: supply, fuse, kill switch and INA226 into a resistor (no drivers)

* **Setup:** VM disconnected from both drivers; 10 Ω power resistor across VM-GND after the INA226. PSU 5.0 V, limit 0.6 A.
* **Procedure:** close SW1; read PSU display, multimeter and `ev=power` together; open SW1.
* **Measure:** `bus_mv` vs multimeter; `ma` vs PSU current (expect about 500 mA); time to 0 after SW1 opens.
* **Stop if:** resistor hotter than expected (2.5 W), INA226 error (`ev=error src=ina226`).
* **Pass:** INA226 within a few % of the meters; SW1 cuts VM.
* **Evidence:** `car.log`, `notes.md`.

### T4: drivers on VM, no motors

* **Setup:** VM to both drivers, resistor removed, motors unplugged. PSU 5.0 V, limit 0.1 A.
* **Procedure:** standby current; `en`; for each channel `w` with only that wheel at 0.5, then −0.5, `zero brake`, `zero coast`; `stby`.
* **Measure:** standby and idle VM current; DC voltage across each AO1-AO2 / BO1-BO2 pair (about ±0.5 × 0.4 × VM); in coast and standby, each output's voltage to GND (should float, WIRING assumption 5).
* **Stop if:** any current above 50 mA with no motor; any output not off in standby.
* **Pass:** every channel follows its own command only; outputs off in standby and coast; brake gives both outputs low.
* **Evidence:** `car.log`, `notes.md` table.

### T5: one motor, wheels off the ground

* **Setup:** chassis on blocks, one motor connected (repeat for each of the four channels). PSU 5.0 V, limit 0.5 A.
* **Procedure:** `en`; stream one wheel at 0.1, 0.2 … 1.0 (= 40 % duty placeholder), 3 s each; then −1.0; `zero brake` / `zero coast` stops.
* **Measure:** steady `ma` per step, `ma_max` at each start, `bus_min_mv`; smallest command that turns the wheel (dead zone, input to O6); motor temperature by touch/IR after the run; which wheel turns and which way (input to O8).
* **Stop if:** current at the PSU limit, motor stalls, smell, heat.
* **Pass:** each channel drives exactly one motor in both directions; currents recorded per duty.
* **Evidence:** `car.log` per channel, `notes.md` with the mapping.

### T6: brief stall (optional)

Only with a PSU limit ≤ 1.0 A (TB6612 continuous rating at VM ≥ 5 V) and a clamped wheel, not a hand.

* **Procedure:** one wheel at 0.5 for 0.5 s, clamped; then at 1.0 for 0.5 s.
* **Measure:** `ma_max`; whether the PSU went into current limit (then stall current is "≥ limit").
* **Stop if:** anything above 0.5 s, smell, driver hot.
* **Pass:** stall current recorded or bounded; if it exceeds 1.0 A, `max_duty_pm` and `trip_ma` must keep it below (O1).
* **Evidence:** `car.log`, `notes.md`.

### T7: four motors on blocks, combined load

* **Setup:** chassis on blocks, four motors. PSU 5.0 V, limit 1.5 A (placeholder; raise only from T5 numbers ×4 plus margin). Fuse one size above the expected peak.
* **Procedure:** one wheel, two, then four together at 0.3, 0.6, 1.0; each 10 s.
* **Measure:** combined `ma` and `ma_max`, `bus_min_mv`, any `ev=boot` (unplanned reset), Wi-Fi not involved.
* **Stop if:** PSU limit reached, unplanned reset, `ev=motor reason=OverCurrent` (trip at `trip_ma`).
* **Pass:** four-wheel current recorded; no resets; fuse value chosen (O1).
* **Evidence:** `car.log`, `notes.md`.

### T8: starts and reversals, slew limit (O7)

* **Setup:** as T7. Rebuild with `slew_per_s` = 2.0 (default), then 5.0, then `f32::INFINITY` (off), lowest first; `max_duty_pm` as T7 allowed.
* **Procedure:** all four 0 → 1.0, hold 2 s, → −1.0, hold 2 s, → 0; five times per setting.
* **Measure:** `ma_max`, `bus_min_mv`, `ev=boot reset=SysBrownOut` or other unplanned resets, logic-analyzer glitches on `STBY`/`HEARTBEAT`; 3V3 with a scope if available.
* **Stop if:** any reset or brownout: keep the last good setting.
* **Pass:** the fastest slew with no reset and a bus minimum with margin is recorded; O7 is "needed, rate X" or "not needed".
* **Evidence:** `car.log` per setting, `reversal.sr`.

### T9: fail-safe under four-motor load (O4)

* **Setup:** as T7, all four at the T7 maximum. Capture A, trigger on `STBY` falling; camera on one wheel (`scripts/cam.sh`) for spin-down.
* **Procedure:** one trigger per run: (a) stop streaming (watchdog), (b) `stby`, (c) `w nan 0 0 0`, (d) `hang high`, (e) `hang low`, (f) RST button, (g) unplug USB (VM stays on), (h) open SW1.
* **Measure:** trigger → `STBY` low (logic analyzer; for (a) from the last `w` line in `car.log` timestamps); `STBY` low → wheel stopped (camera, 33 ms frames); that nothing restarts after the reboot without `en`.
* **Stop if:** any wheel keeps driving after `STBY` falls; `STBY` not low within `tw` + margin.
* **Pass:** every trigger puts both drivers in standby; worst case recorded as the O4 bound; no restart without `en`.
* **Evidence:** one `.sr` and `car.log` per trigger, contact sheet.

### T10: brake vs coast (O5)

* **Setup:** as T9.
* **Procedure:** from 1.0 on all wheels: zero with `zero brake`, zero with `zero coast`, and `stby`; three times each.
* **Measure:** spin-down time (camera), `ma_max` and negative `ma` (regenerated current) during the stop, `bus_mv` peak.
* **Stop if:** bus voltage rises above 13.5 V (TB6612 operating maximum) during braking.
* **Pass:** O5's M4 part decided: method, spin-down time and current. Stopping distance at maximum speed stays with M5.
* **Evidence:** `car.log`, contact sheets.

### T11: thermal run

* **Setup:** as T7; the duty and current T7-T10 accepted.
* **Procedure:** four wheels at the accepted maximum, 10 min, streamed; temperature every 2 min.
* **Measure:** driver IC and motor temperatures, current drift, `bus_min_mv`.
* **Stop if:** driver too hot to touch, smell, current rising steadily.
* **Pass:** temperatures recorded and within the parts' limits with margin; safe operating duty, current and thermal limits written to MILESTONE.md for M5.
* **Evidence:** `car.log`, `notes.md` temperature table.

Not covered by this firmware: M4 test plan item 7 (motor noise while sending Wi-Fi commands). It needs M8's radio firmware with this motor layer; plan it after T11.

## Open questions settled here

| Question ([M6 proposal](../../spec/changes/add-four-wheel-api/proposal.md#open-questions)) | Tests | What decides it |
| --- | --- | --- |
| **O1** current limits, max duty for 1.0 | T5, T6, T7, T11 (T3 calibrates the meter) | Per-channel and combined current vs TB6612 1.0 A/ch and the supply; temperatures → `max_duty_pm`, `trip_ma`, fuse |
| **O4** safety stop → every driver in standby, under load | T1 (core watchdog), T2 (`tw`, RTC watchdog), T9 (all triggers, four motors) | Worst trigger → `STBY` low time in T9 |
| **O5** zero: brake or coast (M4 part) | T4 (pin states), T10 | Spin-down time and stop current; stopping distance is M5 |
| **O7** slew limit for starts and reversals | T8 (T7 for the baseline) | Fastest slew with no reset and margin on `bus_min_mv` |
