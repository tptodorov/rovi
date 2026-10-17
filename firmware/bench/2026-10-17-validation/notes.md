# 2026-10-17: product firmware after task 3.1 (profile validation, deadlines before input)

Firmware `aa89d11` (branch `m6/device-validation`, PR #59). Car on its stand, wheels clear, two power banks,
BLE from the laptop (`experiments/rovi-m8-setpoint-ros/sim/ble_client.py`). Serial log: [car.log](car.log),
client output: [runs.txt](runs.txt). Development evidence, not product acceptance.

| Run | Command | Car log | Observed (Todor) |
| --- | --- | --- | --- |
| - | Flash, boot | `ev=boot reset=ChipPowerOn`, no panic: `Device::new` accepted `Geometry::CAR` with `Config::DEFAULT` | - |
| 1 | `vx 0.1`, 3 s, STOP | Armed 3.1 s at 83 ‰ (about 42 % duty), `ev=stop reason=Stop` | All four forward, stopped |
| 2 | `vx 0.5` 3 s, then silent 3 s | Armed at 417 ‰, `ev=stop reason=LeaseExpired` 4.0 s after arming (about 1 s after the stream) | All four forward, stopped by themselves |

No `ev=error`. Not exercised on the board: the deadline-ordering edge itself (a packet landing on a deadline
before the next tick), unsupported frames, and an invalid profile; those are host-tested.
