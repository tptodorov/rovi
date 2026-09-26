## Remote Controlled Car with 4 mecanum-wheels

### Status

* Hardware parts package: **received** (2026-09-26)

### Car

* 4x mecanum wheels
* 1x car frame
* 4x electric engines
* 2x 2-motor drivers
* 1-2 rechargeable batteries with USB ports
* Raspberry Pi Zero 2 W

### New parts received (2026-09-26)

* 1x **ESP32-S3-N16R8** development board (CORE board, USB Type-C; 2.4 GHz WiFi + Bluetooth; 16 MB flash / 8 MB PSRAM; pin headers not soldered). Label part ID `FBBA0086-001`.
* 5x **TB6612FNG** dual motor driver boards (1 A; Arduino-compatible; often used instead of L298N). Pack quantity: 5 pcs.

### Control Software

The following package is meant to run on Raspberry Pi with Python 3.11.

The 4 motors are controlled via PWM pins connected to the motor drivers.
For each motor, one pin is used to control the direction and another pin to control the speed.

Implemented control methods:

* over Bluetooth PS4 controller directly connected to the car
* network via websockets from any internet host using a web page
* network via Zenoh from the local network using a keyboard
