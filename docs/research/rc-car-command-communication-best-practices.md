# Rovi: RC-car command communication best practices

**Research snapshot:** 2026-10-04

**Status:** research only. This is not an approved architecture decision. The "Implications for Rovi" section contains recommendations. Record accepted changes as ADRs or as M6 spec requirements.

This note compares the M3 design with established practice in hobby RC links (ExpressLRS/CRSF), autopilots (ArduPilot Rover, PX4), ROS 2 teleop, and MAVLink. It covers link-loss failsafes, command semantics, transport, arbitration, emergency stop, security, and timing. Every claim cites a primary source: official docs, specs, or source pinned to a commit. Claims marked **unverified** could not be traced to a primary source.

## M3 design in brief

M3 uses a WPA2 car AP with a TCP binary protocol v1. Each command carries an absolute board-time expiry and a per-session sequence. BLE uses one-byte GATT writes. Both transports feed one 8-slot FIFO, with reject-newest when it is full. Stop takes priority and purges the queue. Each source has a 1 s watchdog lease, and stale commands expire at admission and at dequeue. See the [M3 README](../../experiments/rovi-m3-wifi-queue/README.md) and [results](../../experiments/rovi-m3-wifi-queue/MILESTONE.md#results-2026-10-04).

Bench findings that this note addresses:

* Client Wi-Fi power save delayed downlink by up to ~240 ms.
* The single TCP listener refuses connections during session teardown.
* A 2 s TCP socket timeout, not the watchdog, bounds Wi-Fi silence.
* BlueZ service discovery can exceed the 1 s initial BLE lease.

## 1. Failsafe and link-loss handling

### Conventions

**Receiver-level options** follow three conventions: *no pulses* (cut), *hold last*, and *preset position*.

* ExpressLRS defines `FAILSAFE_NO_PULSES`, `FAILSAFE_LAST_POSITION`, and `FAILSAFE_SET_POSITION`. PWM outputs offer `PWMFAILSAFE_SET_POSITION`, `PWMFAILSAFE_NO_PULSES` ("stop pulsing"), and `PWMFAILSAFE_LAST_POSITION` ([`common.h`](https://github.com/ExpressLRS/ExpressLRS/blob/8c51826de3ae95fa02002d813b120c677bdf122a/src/include/common.h)).
* CRSF signals failsafe by omission. With failsafe type "cut", RC channel frames (`0x16`) stop, and the spec says "it is recommended to wait for 1 second before starting the FC failsafe routine" ([TBS CRSF spec](https://github.com/tbs-fpv/tbs-crsf-spec/blob/b21795a8a76e3370d259e7163a2164678d89bdaf/crsf.md#0x16-rc-channels-packed-payload)).
* FrSky failsafe modes and timings: **unverified** (not checked against FrSky primary docs).

**Hold last is dangerous for a ground vehicle without a supervisor.** PX4 warns that "the vehicle will use the last supplied setpoint until the timeout triggers", so the timeout must be short ([PX4 `COM_RC_LOSS_T`](https://github.com/PX4/PX4-Autopilot/blob/9fda8a51a4952dd6964fb7e2c86e0a4f748f740b/src/modules/commander/commander_params.yaml)).

**Failsafe is a two-stage pattern.** A link timeout triggers a vehicle action (Hold/stop, RTL, or Disarm). The action is configured separately from the timeout.

### Timeouts in practice

| System | Parameter | Default | Meaning |
|---|---|---:|---|
| ExpressLRS servo outputs | `FAILSAFE_ABS_TIMEOUT_MS` | 1000 ms | Failsafe after 1 s without a channels packet, or LQ = 0 ([source](https://github.com/ExpressLRS/ExpressLRS/blob/8c51826de3ae95fa02002d813b120c677bdf122a/src/lib/ServoOutput/devServoOutput.cpp), [docs](https://www.expresslrs.org/hardware/pwm-receivers/)). |
| CRSF flight controller | — | 1 s (recommended) | Wait after channel frames stop ([spec](https://github.com/tbs-fpv/tbs-crsf-spec/blob/b21795a8a76e3370d259e7163a2164678d89bdaf/crsf.md#0x16-rc-channels-packed-payload)). |
| ArduPilot Rover | `FS_TIMEOUT` | 1.5 s | How long a failsafe condition must persist. Source default is 1.5; the [wiki](https://ardupilot.org/rover/docs/rover-failsafes.html) says 1 s. Source wins ([`Parameters.cpp`](https://github.com/ArduPilot/ardupilot/blob/e204ca77a8012342b52af17beb78adaca598113c/Rover/Parameters.cpp)). |
| ArduPilot Rover | `FS_ACTION` | 2 (Hold) | Options: Nothing, RTL, Hold, SmartRTL, Terminate, Loiter (same file). |
| ArduPilot Rover | `FS_GCS_ENABLE` / `FS_GCS_TIMEOUT` | disabled / 5 s | GCS heartbeat loss (same file). |
| ArduPilot Rover | `GUID_TIMEOUT` | 3 s | "vehicle will stop if no updates are received" for velocity/throttle/heading/rate targets (same file). |
| ArduPilot | `RC_OVERRIDE_TIME` | 3 s | After this, MAVLink RC overrides are dropped and RC input resumes ([`RC_Channels_VarInfo.h`](https://github.com/ArduPilot/ardupilot/blob/e204ca77a8012342b52af17beb78adaca598113c/libraries/RC_Channel/RC_Channels_VarInfo.h)). |
| PX4 | `COM_RC_LOSS_T` | 0.5 s | Manual-control (RC or joystick) loss. "Ensure the value is not set lower than the update interval" ([`commander_params.yaml`](https://github.com/PX4/PX4-Autopilot/blob/9fda8a51a4952dd6964fb7e2c86e0a4f748f740b/src/modules/commander/commander_params.yaml)). |
| PX4 | `NAV_RCL_ACT` | 2 (Return) | Options include Hold and Disarm (same file). |
| PX4 | `COM_OF_LOSS_T` / `COM_OBL_RC_ACT` | 1.0 s / 0 (Position) | Offboard setpoint-stream loss (same file). |
| PX4 | `COM_DL_LOSS_T` / `NAV_DLL_ACT` | 10 s / 0 (Disabled) | GCS datalink loss (same file). |
| ROS 2 `diff_drive_controller` | `cmd_vel_timeout` | 0.5 s | Brakes to zero when the last command is older than this ([params](https://github.com/ros-controls/ros2_controllers/blob/145ca309853dd373f563474f11e2c2cd96799ee3/diff_drive_controller/src/diff_drive_controller_parameter.yaml), [source](https://github.com/ros-controls/ros2_controllers/blob/145ca309853dd373f563474f11e2c2cd96799ee3/diff_drive_controller/src/diff_drive_controller.cpp)). |
| ROS 2 `twist_mux` (example config) | `timeout` | 0.5 s | Per input; expired inputs lose arbitration ([config](https://github.com/ros-teleop/twist_mux/blob/a8225e2d9bf0171aac447c246376f0981defd17b/config/twist_mux_topics.yaml)). |
| MAVLink heartbeat | — | 1 Hz; 4–5 missed | Typical on RF links. The rate is "not defined by MAVLink" ([heartbeat protocol](https://mavlink.io/en/services/heartbeat.html)). |

**Takeaways:**

* Direct-control loss timeouts cluster around **0.5–1.5 s**. GCS/telemetry heartbeats use multi-second timeouts.
* Rovi's 1 s lease is within the norm for direct control.

## 2. Command semantics: stream setpoints, don't queue motion

**Continuous control is a stream of absolute setpoints where the latest wins.** None of the reviewed systems queues motion commands for later execution:

* CRSF sends all 16 channels in every `0x16` frame ([spec](https://github.com/tbs-fpv/tbs-crsf-spec/blob/b21795a8a76e3370d259e7163a2164678d89bdaf/crsf.md#0x16-rc-channels-packed-payload)).
* MAVLink `MANUAL_CONTROL` and `RC_CHANNELS_OVERRIDE` carry full axis or channel state ([messages](https://mavlink.io/en/messages/common.html)).
* ROS 2 `cmd_vel` carries a full twist ([diff_drive_controller](https://control.ros.org/rolling/doc/ros2_controllers/diff_drive_controller/doc/userdoc.html)).

**The setpoint stream doubles as the heartbeat.** PX4 Offboard needs a continuous ≥2 Hz stream: "the setpoint messages convey both the signal to indicate that the external source is 'alive', and the setpoint value itself" ([PX4 Offboard](https://docs.px4.io/main/en/flight_modes/offboard.html)).

**Timeliness beats completeness.** The ROS 2 sensor-data QoS profile uses best effort and a small queue because "it's more important to receive readings in a timely fashion, rather than ensuring that all of them arrive." ROS 2 also defines *Deadline*, *Lifespan* ("expired messages are silently dropped") and *Liveliness lease duration* ([ROS 2 QoS](https://github.com/ros2/ros2_documentation/blob/88573f7a5acce6238cd176bfa2e3b4757c4f4058/source/ROS-Framework/interfaces/topics/About-Quality-of-Service-Settings.rst)).

**Stale input is dropped on arrival and timed out in the control loop.** `diff_drive_controller` ignores a received `TwistStamped` older than `cmd_vel_timeout`. It also brakes when the last command ages past the timeout ([source](https://github.com/ros-controls/ros2_controllers/blob/145ca309853dd373f563474f11e2c2cd96799ee3/diff_drive_controller/src/diff_drive_controller.cpp)).

**Discrete, non-idempotent actions are a separate class.** Examples are arm, mode change, and mission items. They use request/acknowledge protocols and are not part of the setpoint stream (e.g. MAVLink command and mission microservices; [MAVLink services](https://mavlink.io/en/services/)).

**Typical rates:**

* ExpressLRS air rates range from 25 Hz to 1000 Hz ([signal health](https://www.expresslrs.org/info/signal-health/)).
* PX4's floor is 2 Hz.
* A timeout should cover several missed updates (MAVLink: 4–5 heartbeats).

## 3. Transport

* **TCP head-of-line blocking.** A single TCP stream delivers in order. One lost segment stalls every later byte until it is retransmitted. QUIC lists avoiding this as a benefit: "only streams with data in that packet are blocked" ([RFC 9000 §13](https://www.rfc-editor.org/rfc/rfc9000#section-13)). For a latest-wins stream, a retransmitted old setpoint has no value and delays the fresh one.
* **Nagle and delayed ACK.** RFC 1122 says a TCP SHOULD use Nagle. Applications MUST be able to disable it, and interactive "small message" traffic often needs it off ([RFC 1122 §4.2.3.4](https://www.rfc-editor.org/rfc/rfc1122#section-4.2.3.4)). Delayed ACK may hold an ACK for up to 0.5 s ([§4.2.3.2](https://www.rfc-editor.org/rfc/rfc1122#section-4.2.3.2)).
  * smoltcp 0.12 enables Nagle by default (`nagle: true`). embassy-net exposes `set_nagle_enabled` ([smoltcp `tcp.rs`](https://docs.rs/crate/smoltcp/0.12.0/source/src/socket/tcp.rs), [embassy-net `tcp.rs`](https://docs.rs/crate/embassy-net/0.8.0/source/src/tcp.rs)). M3 does not call it.
* **embassy-net listener and timeout.** "Incoming connections when no socket is listening are rejected," and multiple listening sockets are needed to accept several connections. `set_timeout` closes the socket "if no data is received for the specified duration" ([embassy-net `tcp.rs`](https://docs.rs/crate/embassy-net/0.8.0/source/src/tcp.rs)). Together these explain M3's `ECONNREFUSED` window and its ~2 s Wi-Fi cap.
* **MAVLink.** Messages are self-contained datagrams with an 8-bit `seq` "used to detect packet loss" ([serialization](https://mavlink.io/en/guide/serialization.html)). They run equally over UDP, TCP, or serial.
* **CRSF.** A frame is `[sync][len][type][payload][CRC8 poly 0xD5]`, at most 64 bytes. The 16 channels are packed into 22 bytes, at 416666 baud on the receiver UART. Link statistics (`0x14`) report link quality separately ([spec](https://github.com/tbs-fpv/tbs-crsf-spec/blob/b21795a8a76e3370d259e7163a2164678d89bdaf/crsf.md)).
* **ESP-NOW.** It is connectionless, with no AP or association. Payloads are up to 250 B (v1) or 1470 B (v2). Encryption uses CCMP with PMK/LMK, for up to 17 encrypted peers (default 7). Send success means only "received successfully on the MAC layer"; application ACKs are the developer's job ([ESP-NOW](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/network/esp_now.html)). It suits a dedicated ESP32 handheld, not a phone.
* **BLE connection parameters.**
  * `connInterval` is 7.5 ms–4.0 s in 1.25 ms steps.
  * `connSupervisionTimeout` is 100 ms–32 s and must exceed `(1 + latency) × subrate × interval × 2` ([Core 5.4 Vol 6 Part B](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/Core-54/out/en/low-energy-controller/link-layer-specification.html)).
  * Apple requires Interval Min ≥ 15 ms and supervision timeout 2–6 s ([QA1931](https://developer.apple.com/library/archive/qa/qa1931/_index.html)). An iPhone link therefore detects loss in ≥2 s, slower than a 1 s application lease.
  * Each write-with-response takes at least one connection event for the ATT response, which limits the command rate. **Unverified:** the exact rate depends on the stack.
* **Wi-Fi power save.**
  * Under modem sleep, "the delay in receiving Wi-Fi data may be the same as the DTIM cycle (minimum power-saving mode) or the listening interval (maximum)". The AP "only caches unicast data" for sleeping stations ([Espressif performance and power save](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-guides/wifi-driver/wifi-performance-and-power-save.html)).
  * esp-radio 1.0.0-beta.1 defaults the SoftAP to `dtim_period: 2` with `beacon_interval: 100` TU, a DTIM cycle of about 205 ms. `dtim_period` can be set from 1 to 10 behind `unstable` ([`ap.rs`](https://docs.rs/crate/esp-radio/1.0.0-beta.1/source/src/wifi/ap.rs), [`mod.rs`](https://docs.rs/crate/esp-radio/1.0.0-beta.1/source/src/wifi/mod.rs)).
  * The bench's ~240 ms downlink delay matches roughly one DTIM cycle plus airtime. **This is an inference; it is not measured.** Power save delays *downlink* to the client: ACKs and the banner. Client-to-car commands are not buffered by the AP (**inference from the Espressif text**).
  * On Linux, NetworkManager can disable power save per profile with `802-11-wireless.powersave = 2` ([NM settings](https://networkmanager.dev/docs/api/latest/settings-802-11-wireless.html)). Phone OS control of client power save: **unverified**.
* **Wi-Fi/BLE coexistence on ESP32-S3.**
  * The radio is time-division multiplexed: modules "cannot use the RF path to transmit or receive at the same time".
  * Espressif rates *SoftAP connected + BLE connected* as **C1, "supported but the performance is unstable"** ([coexistence](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-guides/coexist.html)).
  * In coexistence, Wi-Fi sleeps outside its time slice even with `WIFI_PS_NONE` ([power save](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-guides/wifi-driver/wifi-performance-and-power-save.html)).

## 4. Arbitration between control sources

* **twist_mux: priority plus per-input timeout.** Each input has `timeout` and `priority` (0–255). The highest-priority input that has not expired wins. Expiry uses the receive time (`stamp_ = mux_->now()`), not the sender's clock. *Locks* are Bool topics with a priority that mute all lower-priority inputs. A lock with `timeout > 0` must be republished; if its publisher dies, the lock engages, which fails safe ([locks config](https://github.com/ros-teleop/twist_mux/blob/a8225e2d9bf0171aac447c246376f0981defd17b/config/twist_mux_locks.yaml), [`topic_handle.hpp`](https://github.com/ros-teleop/twist_mux/blob/a8225e2d9bf0171aac447c246376f0981defd17b/include/twist_mux/topic_handle.hpp)).
* **PX4: one manual-control source at a time.** `COM_RC_IN_MODE` selects it. The default is 3, "RC or MAVLink keep first until reboot". Explicit priority modes (5–8) switch to a higher-priority source "as soon as it becomes valid" ([`commander_params.yaml`](https://github.com/PX4/PX4-Autopilot/blob/9fda8a51a4952dd6964fb7e2c86e0a4f748f740b/src/modules/commander/commander_params.yaml)).
* **ArduPilot: identity filter and pilot override.**
  * `MAV_GCS_SYSID` (default 255) sets which MAVLink system IDs may send RC overrides, manual control, and GCS heartbeats ([`GCS.cpp`](https://github.com/ArduPilot/ardupilot/blob/e204ca77a8012342b52af17beb78adaca598113c/libraries/GCS_MAVLink/GCS.cpp)).
  * `RC_OPTIONS` can ignore the receiver or MAVLink overrides, or "Clear MAVLink overrides on any stick input" (bit 14) ([`RC_Channels_VarInfo.h`](https://github.com/ArduPilot/ardupilot/blob/e204ca77a8012342b52af17beb78adaca598113c/libraries/RC_Channel/RC_Channels_VarInfo.h)).
* **None of these interleaves two operators' motion commands.** One source owns control at a time. Others can preempt it by priority or take over explicitly. Stop and locks are the exception: they are accepted from any qualified source.

## 5. Emergency stop, latching, and re-arm

* **PX4 kill switch.** It "immediately stops all motor outputs"; reverting within 5 s restarts the motors. After that "the vehicle will automatically disarm, and you will need to arm it again". The arm switch exists so that "there is an intentional step involved before the motors start" ([PX4 safety](https://github.com/PX4/PX4-Autopilot/blob/9fda8a51a4952dd6964fb7e2c86e0a4f748f740b/docs/en/config/safety.md#kill-switch)).
* **ArduPilot.**
  * The aux functions `MOTOR_ESTOP` (31) and `ARM_EMERGENCY_STOP` (165) call `SRV_Channels::set_emergency_stop` ([`RC_Channel.h`](https://github.com/ArduPilot/ardupilot/blob/e204ca77a8012342b52af17beb78adaca598113c/libraries/RC_Channel/RC_Channel.h)).
  * Rover `ARMING_REQUIRE` (default 1) keeps outputs at minimum until arming checks pass and an explicit arm (stick or GCS) arrives ([`AP_Arming.cpp`](https://github.com/ArduPilot/ardupilot/blob/e204ca77a8012342b52af17beb78adaca598113c/libraries/AP_Arming/AP_Arming.cpp)).
* **ROS twist_mux.** An e-stop is typically a high-priority lock, and a lock with a timeout engages when its publisher dies (see §4).
* **Industrial standards.** IEC 60204-1 defines stop categories and emergency-stop requirements in Clause 9 ([IEC webstore](https://webstore.iec.ch/en/publication/26037)). ISO 13850 covers the emergency stop function. **Unverified (paywalled; full text not checked):** the exact stop category definitions (Cat 0 removes power immediately, Cat 1 is a controlled stop then power removal, Cat 2 is a controlled stop with power kept), and the rule that resetting an e-stop must not by itself restart the machine.
* **Common pattern.** Stop is accepted from any authenticated source on a path that bypasses normal command flow. It *latches*. Motion resumes only after an explicit, deliberate re-arm by the operator, never just because a new movement command arrives.

## 6. Security of local links

* **MAVLink 2 signing.** It uses a 32-byte shared key and a 48-bit truncated SHA-256 over the key and the packet. Timestamps are in 10 µs units and must increase monotonically per (system, component, link) stream. Receivers reject older timestamps and timestamps more than 1 min behind local time. Whether to accept unsigned packets is implementation-defined ([message signing](https://mavlink.io/en/guide/message_signing.html)). The scheme authenticates without encrypting and is independent of the transport.
* **BLE.**
  * LE Secure Connections pairing uses P-256 ECDH.
  * Just Works gives no MITM protection ("Unauthenticated no MITM protection"). Legacy Just Works "provides no protection against eavesdroppers or man in the middle attacks" ([Core 5.4 Vol 3 Part H](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/Core-54/out/en/host/security-manager-specification.html)).
  * M3 BLE is unpaired and unencrypted.
* **Wi-Fi.**
  * WPA3-Personal gives "increased protections from password guessing attempts" through SAE and requires PMF ([Wi-Fi Alliance](https://www.wi-fi.org/discover-wi-fi/security)).
  * ESP32-S3 SoftAP supports `WIFI_AUTH_WPA3_PSK` and `WIFI_AUTH_WPA2_WPA3_PSK`. PMF Required is mandatory for WPA3 SoftAP ([Espressif Wi-Fi security](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-guides/wifi-security.html)).
  * esp-radio exposes `Wpa3Personal` and `Wpa2Wpa3Personal`. Its AP config currently sets PMF `capable: true, required: false` ([`mod.rs`](https://docs.rs/crate/esp-radio/1.0.0-beta.1/source/src/wifi/mod.rs)). **Unverified:** whether esp-radio's WPA3 SoftAP path enforces PMF.
  * Without PMF, spoofed deauthentication can force disconnects. **This is an inference.** On Rovi a forced disconnect stops the car, which is safe but is a denial of service.

## 7. Time and latency

* **Relative, receive-time leases are the norm** for liveness. twist_mux stamps inputs on receipt. PX4 and ArduPilot time from the last received setpoint or heartbeat (sources above).
* **Sender timestamps need clock agreement.**
  * `diff_drive_controller` compares the sender's `header.stamp` with local time, which assumes synced clocks. It replaces a zero stamp with the receive time ([source](https://github.com/ros-controls/ros2_controllers/blob/145ca309853dd373f563474f11e2c2cd96799ee3/diff_drive_controller/src/diff_drive_controller.cpp)).
  * MAVLink signing tolerates 1 min of skew because its timestamps exist for replay protection, not freshness.
* **Teleop latency budgets** (e.g. figures from 3GPP TS 22.186 remote driving) could not be retrieved from a primary source: **unverified**. Use Rovi's own bench targets (p95 ACK RTT ≤100 ms near the board) until product requirements set a budget.

## Implications for Rovi (recommendations)

These recommendations map the practices above onto the M3 design. They are not decisions. Each "change" item needs an ADR or an M6 spec requirement before implementation.

### Keep

* **Per-source watchdog lease, default 1 s, renewed only by valid admitted input.** This sits between PX4's 0.5 s and ArduPilot's 1.5 s, and matches the ExpressLRS and CRSF 1 s. Rejected or malformed input does not renew it, which is stricter than "any packet" and good.
* **Failsafe action = "no pulses".** Deasserting STBY on every driver is the ELRS `NO_PULSES` / CRSF "cut" convention and ArduPilot's Hold intent for a rover. Never adopt "last position" for motion.
* **A stop path that bypasses the queue, purges motion, and latches.** This matches every reviewed system.
* **Session plus sequence replay rejection, and no automatic resend of motion.** This matches MAVLink `seq` and its signing replay rules.
* **Fail closed on an unknown protocol version.** Keep WPA2 as the minimum; never fall back to an open AP.

### Change

1. **Switch movement to latest-setpoint semantics.** Each source gets a one-slot "current setpoint" that newer setpoints overwrite. Streaming the full desired state (later the four wheel or twist values) at a fixed rate, e.g. 20–50 Hz, makes the stream itself the lease renewal, as in PX4 Offboard. Reserve a small FIFO only for discrete, acknowledged commands such as arm/disarm, mode, and config. In an 8-deep FIFO, a backlog replays old intent and adds latency, which is what the stale and `Full` machinery currently works around.
2. **Give control to one owner instead of interleaving sources.** Stop interleaving two controllers' motion in one global order. Adopt one owner (PX4-style "keep first") or a priority mux (twist_mux-style), with explicit takeover. Non-owners may still send **stop** and **ping**. This removes the "opposing commands alternate" behaviour recorded in step 4.
3. **Require an explicit re-arm after any stop, timeout, or disconnect.** A latched stop should clear only on an explicit `arm` command from the owner, not when the consumer observes it, and not on the first new movement command (PX4, ArduPilot `ARMING_REQUIRE`). Boot should start disarmed.
4. **Make the watchdog the only liveness authority on Wi-Fi.** Set the TCP socket timeout above `ROVI_WATCHDOG_MS` (or derive it from the watchdog), so that the configured lease, not the 2 s socket timeout, governs Wi-Fi.
5. **Remove the reconnect refusal window.** Keep a second socket listening, or re-arm `accept` before tearing the old session down. Answer an extra client with a "busy" line and close it, instead of returning RST. This follows embassy-net's documented "no listening socket = rejected" behaviour.
6. **Disable Nagle on the board (`set_nagle_enabled(false)`) and use `TCP_NODELAY` on clients.** Many small ACK lines are the case RFC 1122 says to exempt.
7. **Set the SoftAP DTIM to 1** (esp-radio `dtim_period`, `unstable`). This halves the bound on power-save-buffered downlink delay (~205 ms → ~102 ms) when clients wake per DTIM. Document client power save off for laptop testing (NetworkManager `powersave=2`). Then re-measure on a dedicated client radio.
8. **Keep BLE links open across lease expiry.** On lease expiry, stop and disarm, but keep the BLE connection. Closing it forces BlueZ to rediscover services, which exceeded 1 s on the bench. The lease should matter only while armed, so an idle connected but disarmed client is safe.

### Consider

* **UDP for the Wi-Fi setpoint stream.** It avoids TCP head-of-line blocking, Nagle, and retransmitted stale setpoints. Use one datagram per setpoint with session, sequence, and latest-wins, as MAVLink does over UDP. Keep TCP, or a request/ACK over UDP, for discrete commands and discovery. Decide in M6 or M7 with a measured comparison under loss.
* **Drop absolute board-time deadlines for streamed setpoints.** Liveness and freshness would then come from the receive-time lease plus sequence ordering, as in twist_mux. The client clock anchor and its power-save fragility go away, and BLE and Wi-Fi share one rule. If a freshness horizon is still wanted, carry a relative "valid for N ms after receipt".
* **Application-level message authentication across both transports.** For example, an HMAC with a monotonic counter in the MAVLink-signing style, or BLE LE Secure Connections with bonding plus encryption required on the write characteristic. Today BLE is the weakest link: it is unauthenticated, and WPA2 does not cover it.
* **WPA3-Personal or WPA2/WPA3 transition with PMF.** First verify PMF enforcement in esp-radio and iPhone/laptop support.
* **BLE parameters.** Request a 15–30 ms connection interval within Apple's rules. Use write-without-response plus sequence for streamed setpoints, and keep write-with-response for discrete commands. Rely on the 1 s application lease, not the BLE supervision timeout (2–6 s on Apple).
* **Concurrent BLE and Wi-Fi control sessions.** Espressif rates SoftAP-connected plus BLE-connected as C1 (unstable). Treat one active control transport at a time as the default, and test coexistence under video load in M7.
* **ESP-NOW for a future dedicated handheld remote.** It needs no association and offers CCMP encryption. It does not suit phones.
* **Optional two-stage failsafe.** A short lease timeout stops the car (`STBY` off). A longer one disarms, or lower the lease. In both cases re-arm stays explicit. Measure stopping distance on the real car (M4/M5) before choosing the lease value.

## Sources

Primary sources only. They are linked inline with parameter names and defaults where relevant. Source-code links are pinned to commits fetched on 2026-10-04:

* ArduPilot `e204ca77`, PX4 `9fda8a51`, ExpressLRS `8c51826d`, TBS CRSF spec `b21795a8`, twist_mux `a8225e2d`, ros2_controllers `145ca309`, ros2_documentation `88573f7a`.
* Crate sources: esp-radio 1.0.0-beta.1, embassy-net 0.8.0, smoltcp 0.12.0 (versions from the M3 `Cargo.lock`).
* Standards: RFC 1122, RFC 9000, Bluetooth Core 5.4 (Vol 6 Part B, Vol 3 Part H), Apple QA1931, Wi-Fi Alliance security page, IEC 60204-1 webstore abstract.
* Espressif ESP-IDF (latest, ESP32-S3) docs.

Not verified from primary sources: FrSky failsafe specifics, IEC 60204-1/ISO 13850 clause text, 3GPP teleop latency figures, and phone OS power-save control.
