# Rovi

Rovi is a modular robotics hardware and software platform. Its layers support different built devices and the applications that use them.

## Language

**Hardware platform**:
The physical chassis and electronics that provide the base for a built device.

**Device**:
A particular robot built on Rovi's hardware platform, with capabilities and supported commands defined for that build.
_Avoid_: Robot (when a specific build is meant)

**Car**:
The first device: a 4-wheel mecanum car.

**Motor control**:
The capability to control a device's motors independently, without defining the higher-level movement behavior of the device.

**Wheel command**:
A signed, normalised drive request for one wheel, produced by the device layer and consumed by motor control.

**Device control API**:
The set of commands a device supports and the meaning of those commands. Each device defines its API from its own capabilities.

**Device command**:
An operation requested through a device's control API.

**Setpoint**:
A device command that states the full desired motion of the device (for the car, a body velocity), where only the latest one matters.
_Avoid_: Drive command (for continuous motion)

**Control interface**:
A device-facing boundary that turns received control input into commands supported by that device.

**Transport**:
A communication path that carries control input between a client and a device's control interface. The transport does not define the meaning of device commands.

**Application**:
Software behavior that uses a device's control API, such as teleoperation or autonomous driving.

### Control and safety

**Controller**:
A client that sends control input to a device, for example an iPhone app, the development laptop, or a ROS 2 graph.
_Avoid_: Remote, operator (for the software client)

**Source**:
The transport through which a controller's input arrived (BLE, Wi-Fi or ROS 2).

**Session**:
One connection of one controller, from connect until disconnect or timeout. Input from an earlier session is never valid in a later one.

**Owner**:
The single controller allowed to command the device at a given time. The first controller to connect becomes the owner.

**Lease**:
The owner's right to keep commanding the device, renewed only by its valid commands or pings.

**Watchdog timeout**:
The safety event raised when a lease expires without renewal.

**Safety stop**:
The latched state in which all motion is stopped and motor drivers are put in standby, entered on stop, timeout or controller loss.
_Avoid_: E-stop (it is not a certified emergency stop)

**Stale command**:
A device command whose deadline passed before it was applied. It is discarded, never applied late.

**Car AP**:
The device's own Wi-Fi network, which controllers join directly, with no external router.

### Process

**Milestone**:
A numbered unit of exploratory work (M1, M2, …) with a goal, risks covered, a test plan and acceptance criteria.

**Spike**:
A short, throwaway feasibility investigation (S1, S2, …) whose output is a recorded answer, not kept code.

**Bench**:
The physical test setup with the real board, and the procedures run on it. Only bench evidence proves hardware behaviour.

**Board-only**:
A milestone run on the bare ESP32-S3 board, with no motor hardware connected.

**Acceptance**:
A milestone's checklist. An item is ticked only after it has been observed on real hardware.

## Abbreviations and technical terms

### Hardware and electronics

| Term | Meaning |
| --- | --- |
| **ESP32-S3** | Espressif's dual-core Xtensa LX7 microcontroller with Wi-Fi and BLE; Rovi's controller chip. |
| **N16R8** | The board's module variant: 16 MB flash, 8 MB PSRAM. |
| **SoC** | System on chip. |
| **MCU** | Microcontroller unit. |
| **Xtensa / LX7** | The ESP32-S3's CPU architecture; Rust targets it through the `esp` toolchain. |
| **RISC-V** | The CPU architecture of other ESP32 chips (e.g. ESP32-C3). |
| **ESP32-C3** | A single-core RISC-V Espressif chip; nano-ros's only supported ESP32. |
| **PSRAM** | External pseudo-static RAM on the module. |
| **RAM / SRAM** | On-chip working memory. |
| **GPIO** | General-purpose input/output pin (e.g. GPIO48 drives the onboard RGB LED). |
| **PWM** | Pulse-width modulation, used to set motor speed. |
| **MCPWM** | The ESP32's motor-control PWM peripheral. |
| **RMT** | The ESP32's remote-control peripheral, used to drive the WS2812 LED. |
| **WS2812** | The addressable RGB LED on the board. |
| **RGB LED** | The onboard tri-colour LED used as the board-only indicator. |
| **UART** | Serial port. |
| **USB-C / CH343** | The board's USB connectors; CH343 is the USB-to-UART bridge chip. |
| **JTAG** | Debug interface (USB-Serial-JTAG on the ESP32-S3). |
| **BOOT / RST** | Board buttons for download mode and reset. |
| **TB6612FNG** | Dual H-bridge motor driver IC used for the motors. |
| **STBY** | The motor driver's standby input; deasserting it disables the outputs. |
| **AIN1/AIN2, BIN1/BIN2, PWMA/PWMB, AO1/AO2, BO1/BO2** | TB6612FNG direction inputs, speed inputs and motor outputs for channels A and B. |
| **VM / VCC / GND** | Motor supply, logic supply, ground. |
| **DRV8833, L298N** | Alternative motor driver ICs mentioned for comparison. |
| **TT motor** | The small yellow geared DC hobby motor. |
| **Mecanum wheel** | A wheel with angled rollers that lets a 4-wheel car move sideways and rotate in place. |
| **Encoder** | A wheel-speed sensor; needed for closed-loop control. |
| **Open-loop / closed-loop** | Driving by commanded power only, vs correcting from measured speed. |
| **Inverse kinematics** | Converting a body velocity into the 4 wheel speeds. |
| **Brownout** | Supply voltage dropping low enough to reset the chip. |
| **LDO** | Low-dropout voltage regulator. |
| **PSU** | Power supply unit. |
| **IC / PCB** | Integrated circuit / printed circuit board. |
| **ERC** | KiCad's electrical rules check. |
| **RTC** | The real-time clock domain of the chip. |
| **eFuse** | One-time-programmable chip configuration bits. |

### Radio and networking

| Term | Meaning |
| --- | --- |
| **BLE / LE** | Bluetooth Low Energy. |
| **GATT** | BLE's attribute protocol layer: services and characteristics a client reads and writes. |
| **ATT** | BLE attribute protocol; its error codes (e.g. `0x13` Value Not Allowed) reject writes. |
| **UUID** | 128-bit identifier of a BLE service or characteristic. |
| **Connection interval** | How often a BLE central and peripheral exchange packets. |
| **LE Secure Connections** | BLE pairing mode with ECDH key exchange. |
| **MITM** | Man-in-the-middle attack. |
| **ECDH** | Elliptic-curve Diffie–Hellman key exchange. |
| **AP** | Wi-Fi access point; the car runs one (see **Car AP**). |
| **STA** | Wi-Fi station (client) mode; excluded for now. |
| **AP+STA** | Running access point and station at once. |
| **SSID** | Wi-Fi network name (`Rovi M3`). |
| **WPA2 / WPA3** | Wi-Fi security protocols; WPA2-Personal is used. |
| **SAE** | WPA3's password-based handshake. |
| **PMF** | Protected management frames (required by WPA3). |
| **PMK** | Pairwise master key derived from the passphrase. |
| **CCMP** | WPA2's AES-based encryption. |
| **DTIM** | Delivery traffic indication message: how often the AP wakes power-saving clients. |
| **TU** | Time unit (1.024 ms), used for beacon intervals. |
| **Power save** | A client Wi-Fi mode that sleeps between beacons, adding downlink delay. |
| **Coexistence** | Wi-Fi and BLE sharing one radio. |
| **Wi-Fi Direct / P2P** | Peer-to-peer Wi-Fi without an AP; out of scope. |
| **ESP-NOW** | Espressif's connectionless peer-to-peer radio protocol. |
| **IP / DHCP / DNS** | Internet protocol / automatic address assignment / name lookup; the car AP uses static IPs without DHCP or DNS. |
| **TCP** | Reliable, ordered stream transport; used by M3. |
| **UDP** | Unreliable datagram transport; planned for M8. |
| **Head-of-line blocking** | A fresh TCP message waiting behind a lost or retransmitted older one. |
| **Nagle** | TCP's small-packet batching algorithm, which adds latency for small messages. |
| **ACK** | Acknowledgement. |
| **RTT** | Round-trip time. |
| **p95** | 95th-percentile value of a measurement. |
| **MTU** | Maximum transmission unit (packet size). |
| **ECONNREFUSED** | The OS error for a refused TCP connection. |
| **EOF / RST** | A TCP connection's orderly close / abort. |
| **LEB128** | Variable-length integer encoding. |
| **MAC** | Hardware network address. |
| **RF** | Radio frequency. |

### Firmware and tooling

| Term | Meaning |
| --- | --- |
| **`no_std`** | Rust without the standard library, for bare-metal targets. |
| **Embassy** | Async Rust framework for embedded systems (executor, time, networking). |
| **embassy-net** | Embassy's network stack, built on smoltcp. |
| **embassy-sync** | Embassy's synchronisation primitives (channels, mutexes). |
| **smoltcp** | Small TCP/IP stack used under embassy-net. |
| **esp-hal** | Espressif's Rust hardware abstraction layer. |
| **HAL** | Hardware abstraction layer. |
| **esp-radio** | Espressif's Rust Wi-Fi/BLE driver. |
| **esp-rtos** | Espressif's Rust scheduler that runs Embassy and the radio. |
| **trouble-host (TrouBLE)** | Embassy's Rust BLE host and GATT library. |
| **esp-println / defmt** | Two logging approaches: formatted text vs compact deferred formatting. |
| **ESP-IDF** | Espressif's C SDK (FreeRTOS-based); Rovi doesn't use it. |
| **FreeRTOS** | Real-time OS used by ESP-IDF. |
| **RTOS** | Real-time operating system. |
| **espup** | Installer for the Rust Xtensa toolchain. |
| **espflash** | Tool to flash and monitor ESP32 boards. |
| **ELF** | The compiled firmware binary format. |
| **RWX segment** | A linker warning about a memory segment that is readable, writable and executable. |
| **FFI** | Foreign function interface (calling C from Rust). |
| **Sans-IO** | A protocol library that does no I/O itself, so any runtime can drive it. |
| **FIFO** | First-in first-out queue (M3's shared command queue). |
| **JSONL** | One JSON object per line; the client's evidence log format. |
| **QEMU** | Machine emulator. |
| **Nix / nix-shell** | Reproducible package manager / its dev shell, used for the toolchain. |
| **RTK** | CLI proxy used by agents to condense command output. |
| **KiCad** | PCB and schematic design tool. |
| **OpenSpec** | The spec-driven change-proposal process used for product work. |
| **ADR** | Architecture decision record (`docs/adr/`). |
| **RFC** | Request for comments (an IETF standard, or a project design proposal). |
| **CI** | Continuous integration. |
| **PR** | Pull request. |

### ROS and robotics

| Term | Meaning |
| --- | --- |
| **ROS 2** | Robot Operating System 2: middleware and tools for robot software. |
| **Jazzy / Humble / Kilted** | ROS 2 distribution releases. |
| **Node / topic** | A ROS 2 process / a named publish-subscribe channel. |
| **ROS graph** | The set of nodes and topics ROS 2 tools can discover. |
| **RMW** | ROS middleware layer: the pluggable transport under ROS 2 (DDS or Zenoh). |
| **DDS** | Data Distribution Service, ROS 2's default middleware standard. |
| **QoS** | Quality-of-service settings (reliability, deadline, lifespan). |
| **CDR** | The binary serialisation format of ROS 2 messages. |
| **micro-ROS** | ROS 2 for microcontrollers, connected through an agent. |
| **XRCE-DDS** | DDS for extremely resource-constrained environments; micro-ROS's client–agent protocol. |
| **Agent** | The micro-ROS process on a ROS 2 computer that relays microcontroller clients into DDS. |
| **nano-ros** | A Rust `no_std` ROS 2 client with XRCE, Zenoh and Cyclone DDS backends (S1). |
| **Zenoh** | A pub/sub/query protocol; ROS 2 can use it via `rmw_zenoh`. |
| **zenoh-nostd** | Eclipse's pure-Rust, Embassy-native Zenoh implementation (S2). |
| **rmw_zenoh / rmw_zenohd** | ROS 2's Zenoh middleware / its router daemon. |
| **Router** | The Zenoh process that ROS 2 nodes and the car connect to. |
| **Key expression** | A Zenoh resource name; `rmw_zenoh` encodes domain, topic, type and type hash in it. |
| **Liveliness token** | A Zenoh announcement `rmw_zenoh` uses to make a node and its topics visible in the ROS graph. |
| **Attachment** | Per-message metadata `rmw_zenoh` requires (sequence number, timestamp, gid). |
| **gid** | Globally unique ID of a ROS 2 publisher. |
| **Type hash (RIHS01)** | ROS 2's hash identifying a message type's definition. |
| **Twist / TwistStamped** | ROS 2 velocity message: linear x/y/z and angular x/y/z, optionally with a timestamp. |
| **cmd_vel** | The conventional ROS 2 velocity-command topic. |
| **ros2_control** | ROS 2's framework for controllers and hardware interfaces. |
| **mecanum_drive_controller** | The ros2_control controller that converts Twist into 4 wheel velocities. |
| **twist_mux** | ROS 2 node that selects among several velocity sources by priority. |
| **Nav2** | ROS 2 navigation stack. |
| **SLAM** | Simultaneous localisation and mapping. |
| **Odometry** | Position estimate from wheel motion. |
| **Teleoperation (teleop)** | Driving a device remotely by a human. |
| **RC** | Radio control (hobby remote control). |
| **ExpressLRS (ELRS) / CRSF** | An open RC radio link / its serial frame protocol. |
| **FrSky** | An RC radio manufacturer. |
| **Failsafe** | What a vehicle does when its control link is lost. |
| **ArduPilot / PX4** | Open-source autopilots, including ground-rover support. |
| **MAVLink** | Messaging protocol used by ArduPilot and PX4. |
| **GCS** | Ground control station. |
| **RTL** | Return to launch (an autopilot failsafe action). |
| **IEC 60204-1 / ISO 13849 / ISO 13850** | Machinery safety standards covering stop categories and emergency stop. |
