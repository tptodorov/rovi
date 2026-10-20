# iPhone BLE client proposal

## Why

Rovi's [vision](../../../docs/VISION.md) includes an iPhone controller, but the accepted product has only laptop clients. Propose the first phone interface against the existing BLE API, with explicit safety behavior and a device verification plan for [roadmap Q9/Q18/Q19](../../../docs/PLAN.md#open-questions-and-decisions).

## What Changes

* Propose a native SwiftUI/CoreBluetooth iPhone application: foreground discovery, explicit car selection/connect, visible capabilities and last-reported Status.
* Connect disarmed; require an explicit ARM action with a fresh zero setpoint before six gentle press-and-hold directions become usable. Release/cancel clears intent; STOP latches local motion off until a new ARM and a new touch.
* Bound input age, streaming stalls and cleanup. Leaving the active foreground, locking, Bluetooth/permission loss or disconnection ends the session without automatic reconnect, re-arm or motion replay.
* Verify codecs/state transitions with fakes and UI tests separately from real iPhone/normal-firmware BLE, lifecycle and supervised wheel checks. Q18/Q19 remain open until measured on the selected phone.

## Capabilities

### New Capabilities

* `iphone-ble-teleoperation`: iPhone discovery, ownership, explicit arming, touch control, interruption handling and honest feedback through the existing BLE interface.

### Modified Capabilities

None. [Car control API](../../specs/car-control-api/spec.md), [motor control](../../specs/motor-control/spec.md) and [laptop teleoperation](../../specs/laptop-teleoperation/spec.md) retain their accepted contracts.

## Impact

Future implementation would live under `apps/iphone/`, using Apple's SDKs without a new third-party runtime dependency. Existing normal product firmware (default or `ros`, never `motor-bench`), GATT UUIDs and codecs remain unchanged. BLE connection itself claims ownership and switches off the car AP; discovery alone does not. A missing advertisement cannot distinguish another owner from power/range issues.

This PR contains planning Markdown and a regenerated specification reader only: no app scaffold, Swift, firmware, flashing or device/network tests. Wi-Fi diagnostics, UDP/ROS clients, station mode, BLE credential provisioning (Q21), compact setpoints, pairing/authentication changes, background driving, joystick/mixed-axis control and App Store/TestFlight distribution are outside this proposal.

## Proposal acceptance

**DRAFT — proposed scope, not accepted for implementation.** Todor authorized specification authoring and a draft PR only. No implementation acceptance is recorded. Merging this documentation PR alone does not authorize implementation; Todor must explicitly accept this concrete proposal under the [product gates](../../../docs/SPECIFICATION.md#workflow-and-gates). Task 1.1 remains unchecked until that evidence exists. Physical runs need their own announced batch and Todor's go.

Review the [design and OPEN questions](design.md), [proposed requirements](specs/iphone-ble-teleoperation/spec.md), [tasks](tasks.md) and [verification map](verification.md). No future implementation or physical acceptance task is complete.
