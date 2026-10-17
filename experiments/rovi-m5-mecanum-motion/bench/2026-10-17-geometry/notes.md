# M5 measured-geometry flash and boot — 2026-10-17

Firmware source revision: `7886520`, non-ROS release build. Todor authorized flashing and checking boot only, with the car on its stand.

* [Flash output](flash.txt): completed successfully on ESP32-S3 revision v0.2, 16 MB flash, MAC `14:c1:9f:cd:71:78`.
* [Boot capture](boot.log): `ev=boot ver=0.1.0 reset=ChipPowerOn`, no panic or error observed in the 12-second capture. The monitor was intentionally stopped by its timeout (exit 124).
* [BLE capabilities](capabilities.txt): radius approximately 0.030 m, centre projection sum approximately 0.1275 m, MTU 247. Status was Claimed and disarmed during the information read; the client then disconnected.

No setpoint or ARM command was sent during the flash/boot check. The speed-normalization parameter remains a placeholder; the prior wheel-speed counts belong to the earlier firmware geometry.

## Gentle forward verification on the stand

After a separate announcement and Todor's "go", ran `drive --vx 0.1 --seconds 5` (approximately 45% duty). [Client output](gentle-forward-client.txt): 101 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error. Todor confirmed all four wheels started, turned forward and stopped cleanly. Pass.

Floor motion and loaded speed remain untested.
