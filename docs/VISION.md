# Rovi vision

Rovi is a modular robotics hardware + software platform for 4-wheel-drive vehicles, built one layer at a time:

1. **Hardware platform** — a 4-wheel-drive chassis and electronics that can independently drive 4 wheels.
2. **Low-level control API** — firmware exposes the built device's control capabilities. For the mecanum car, it drives the 4 wheels independently, using as little CPU as possible so most of the chip's budget stays available for higher-level behavior. The API and supported commands are defined for the device being built; Rovi does not require every device to share one universal command set. The car expects a valid command or controller ping within a configurable interval. If the interval expires, firmware raises a timeout event and deasserts the standby signals on every motor driver.
3. **Applications and control interfaces** — applications use the device's control API. Control is available over both Bluetooth Low Energy (BLE) and Wi-Fi network interfaces. Each interface receives input signals, translates them into commands for the current device, and enqueues them in a shared command queue for processing by the system. The initial BLE clients are an iPhone and the development laptop, and each should be able to issue the current device's supported commands. The first device is a 4-wheel mecanum car; its initial application is teleoperation with a video stream. A future application could be autonomous driving; a future *platform* could be a boat or a flying device, reusing layers 1-2 where the hardware and physics allow.

Each layer should stay usable without the layers above it. The reusable motor-control layer should expose independent wheel control without baking in mecanum-specific assumptions; device-specific commands and application behavior belong above that layer. This keeps lower layers repurposable while allowing each built device to expose the controls that make sense for its hardware.

For current status and the roadmap, see [`PLAN.md`](PLAN.md). For how development work actually happens, see the root [`README.md`](../README.md).
