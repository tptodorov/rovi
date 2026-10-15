# M4 bench test plan

Wiring: [WIRING.md](WIRING.md). Firmware: the M8 firmware with motor outputs (`../rovi-m8-setpoint-ros/firmware`). Client: `sim/ble_client.py` (BLE, no Wi-Fi join needed).

## Safety rules

* **Wheel off the ground** (motor loose on the table, or the chassis on a box). Floor driving is M5.
* **Your hand near the power bank cable.** Unplugging it stops the motor whatever the firmware does.
* Unplug the power bank before changing any wiring.
* **Stop at once** on smell, heat, a reset you did not cause, or the motor turning when the car is not armed.

## Setup

```sh
cd experiments/rovi-m8-setpoint-ros
(source ~/export-esp.sh; PATH=~/.rustup/toolchains/esp/bin:$PATH; cd firmware
 ROVI_WIFI_PASSWORD="$(< ~/.config/rovi/m3-wifi-pass)" cargo build --release --target xtensa-esp32s3-none-elf -Zbuild-std=core,alloc)
PORT=/dev/serial/by-id/usb-1a86_USB_Single_Serial_5C85037678-if00
ELF=firmware/target/xtensa-esp32s3-none-elf/release/rovi-m8-firmware
espflash flash --non-interactive --port $PORT $ELF
B=../rovi-m4-four-motor-safety/bench/$(date +%F)-step1; mkdir -p $B
espflash monitor --port $PORT --non-interactive --elf $ELF | tee $B/car.log   # its own terminal
```

`espflash monitor` resets the board, so the log starts at `ev=boot … reset=…`. Every BLE command below runs in the repo nix shell (`nix-shell` at the repo root) from `experiments/rovi-m8-setpoint-ros`.

## Step 1: one driver, one motor (done 2026-10-14, on M1's pins; the firmware now uses the ribbon pin map)

### S1: no motor power

Power bank unplugged, motor connected. A multimeter between `STBY` (driver R4 or GPIO4) and GND.

1. After boot: `STBY` reads 0 V.
2. `python3 sim/ble_client.py drive --vx 0.5 --seconds 5`: `STBY` reads about 3.3 V while it streams, 0 V after the STOP.
3. `python3 sim/ble_client.py drive --vx 0.5 --seconds 2 --quiet 3`: `STBY` drops to 0 V about 1 s after the stream ends (lease expiry), before the STOP.

**Pass:** `STBY` high only while armed and streaming. Log: `ev=state state=Armed wheels_pm=250,…` then `ev=stop reason=Stop` (or `LeaseExpired`).

### S2: one motor, both directions

Plug the power bank in.

1. `drive --vx 0.25 --seconds 3`, then `--vx 0.5`, `--vx 1.0` (10 %, 20 %, 40 % duty). Note the smallest that turns the wheel.
2. `drive --vx -0.5 --seconds 3`: the motor turns the other way.
3. Note the direction for forward (it becomes the FL mapping and inversion).

**Pass:** the motor follows speed and direction; it stops after every run.

### S3: stops under load

With the motor turning (`--vx 1.0`):

| Trigger | How |
| --- | --- |
| STOP | `drive --vx 1.0 --seconds 3` (ends with STOP) |
| Lease expiry | `drive --vx 1.0 --seconds 2 --quiet 3` |
| Disconnect | `drive --vx 1.0 --seconds 10`, then Ctrl-C the client |
| Reset | `drive --vx 1.0 --seconds 10`, press the board's RST button |
| ESP32 unpowered | `drive --vx 1.0 --seconds 10`, unplug the board's USB (power bank stays in) |

**Pass:** every trigger stops the motor within about a second (lease expiry: 1 s after the last setpoint), and after a reset or replug the motor stays off until a client arms again. Log: the matching `ev=stop reason=…`, and `ev=boot reset=…` after a reset.

### S4: radio next to a turning motor

`drive --vx 1.0 --seconds 60`, then `python3 sim/ble_client.py info`.

**Pass:** no unplanned `ev=boot`, no BLE disconnect while the motor runs.

### S5: short thermal run

`drive --vx 1.0 --seconds 300`. Touch the driver chip and the motor after.

**Pass:** warm at most; nothing too hot to hold a finger on.

## Step 2: four motors (after rewiring per WIRING.md)

The firmware already drives all four bridges. Each test is announced and started on your "go".

1. **Which way each wheel turns:** `drive --vx 0.5 --seconds 3` (all four should turn forward). Note each wheel that turns backwards; set `invert: true` for it in `CHANNELS`, reflash, repeat until all four turn forward.
2. **Mecanum pattern:** `--vy 0.5` (strafe left: FL and RR backwards, FR and RL forward) and `--wz 1.0` (turn left: left wheels backwards, right forward). Confirms the mapping.
3. **All four together, increasing:** `--vx 0.25`, `0.5`, `1.0`, then `--vx 1.0` followed at once by `--vx -1.0` (reversal). Pass: no unplanned `ev=boot` (no brownout reset), the power bank keeps supplying.
4. **Stops with four turning:** STOP, lease expiry, RST, ESP32 unplugged, as S3.
5. **Thermal:** 5 min at `--vx 1.0`; drivers and motors at most warm.

## Evidence

`bench/<date>-step<N>/`: `car.log`, `notes.md` (firmware commit, readings, pass/fail, the forward direction and smallest turning speed), a photo of the setup.
