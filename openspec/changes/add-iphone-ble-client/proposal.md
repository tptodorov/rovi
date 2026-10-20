# iPhone BLE web controller proposal

## Why

Rovi's [vision](../../../docs/VISION.md) includes an iPhone controller, but the accepted product has only laptop clients. Todor has Linux, no Mac or Apple Developer Program membership, and prefers no subscription. A browser controller can use the existing BLE API without building or signing a new iOS app; see [roadmap Q9/Q18/Q19](../../../docs/PLAN.md#open-questions-and-decisions).

## What Changes

* Target an HTTPS web controller opened in the free Bluefy iPhone browser, selected by Todor in this conversation. Standard Safari is not a supported BLE host. Develop and test the web code from Linux; no Xcode, native scaffold or signing workflow.
* Use browser-mediated explicit device selection/connect, visible capabilities and last-reported Status. Connect disarmed; require fresh zero then explicit ARM before six gentle press-and-hold directions become usable.
* Buttons are the selected first input method. Keep input interpretation separate from the bounded motion/session policy so a later accelerometer/tilt input can use the same limits, ARM, STOP and expiry rules. Sensor control, sensor permissions and calibration are deferred, not implemented in v1.
* Bound hold age, stream stalls and cleanup. Page/host interruption or link failure clears motion; no automatic reconnect, re-arm or motion replay. Validate Bluefy's lifecycle behavior on the actual phone before accepting control.
* Todor confirms the phone can send the 72-byte payload. Retain runtime failure handling and separate browser/firmware integration evidence from that user-reported phone capability; no new MTU measurement is claimed. Streaming-rate Q19 remains open.

## Capabilities

### New Capabilities

* `iphone-ble-teleoperation`: iPhone browser discovery/selection, ownership, explicit arming, touch control, interruption handling and honest feedback through the existing BLE interface.

### Modified Capabilities

None. [Car control API](../../specs/car-control-api/spec.md), [motor control](../../specs/motor-control/spec.md) and [laptop teleoperation](../../specs/laptop-teleoperation/spec.md) retain their accepted contracts.

## Impact

Future implementation would be static HTML/CSS/JavaScript under `apps/iphone/`, using Web Bluetooth in Bluefy with no third-party runtime scripts or command server. Propose repository GitHub Pages hosting; verify the final origin/HTTPS during delivery. Existing normal product firmware (default or `ros`, never `motor-bench`), GATT UUIDs and codecs remain unchanged. BLE connection itself claims ownership and switches off the car AP; a picker alone does not claim. Missing advertisements do not prove another owner exists.

Specification PR #95 contains planning Markdown only; the button controller lands in a stacked implementation PR under `apps/iphone/`. Neither publishes hosting, runs device/network tests or changes firmware. Native iOS build/signing, Wi-Fi diagnostics, station mode, BLE credential provisioning (Q21), compact setpoints, pairing/authentication changes, background driving, mixed-axis controls and sensor driving are outside v1.

## Proposal acceptance

**Accepted for button-based implementation (2026-10-20).** In this conversation Todor answered the explicit proposal question with: “Accept proposal and implement buttons”. This accepts the revised Bluefy requirements in PR #95 at `b80e673a085d84c6d6dcd87c031194b5e7912bf4`, with accelerometer control deferred. Task 1.1 records that evidence. Product/phone/wheel acceptance remains pending; no deployment or physical test is claimed or authorized by this record. Parent retains merge coordination. Implementation proceeds on the stacked `feat/iphone-ble-web` branch.

Review the [design/OPEN questions](design.md), [requirements](specs/iphone-ble-teleoperation/spec.md), [tasks](tasks.md) and [verification map](verification.md). Implementation progress and evidence are tracked there; physical acceptance remains pending.
