# Verification

Implementation authorized by Todor's “do it” on 2026-10-28. No motors or ARM are needed for this change's phone check.

## Host evidence

* `cd apps/iphone && rtk npm test`: **63 passing** Node tests, including 13 network tests. The first implementation run reproduced unsupported Network/absent mode behavior before the feature was added. A subsequent delayed-GATT test failed before zero writes were interleaved between confirmation reads; it now passes at 125 ms round trips without relaxing the existing 200 ms stream guard or one-second control deadline.
* `cd apps/iphone && rtk npm run build`: builds the pinned module graph.
* Built-page Playwright/Chromium: **43 passing** checks, including six new network cases. Existing direction, STOP, physical-control, responsive-layout and cached-release checks remain green.
* A firmware-path review found ATT writes are rejected on mode refusal. Updating the fake to reject reproduced a failure (client disconnected instead of explaining refusal). Fresh Status now distinguishes rejection from an unconfirmed transport failure; a failed write cannot confirm an already-pending mode.
* Network checks cover strict eight-byte decoding, optional/legacy characteristics, notifications, malformed metadata, armed/busy/invalid guards, Home and Car network requests, fresh matching confirmation, refusal, stalled operations, old callbacks and zero-only writes. Browser tests verify displayed state, last-reported labels, action guards and refusal guidance.

The no-motion hardware soak requested alongside this feature has [separate evidence](../../../firmware/bench/2026-10-28-home-coexistence-buffered/notes.md). It exercised existing firmware, not this phone implementation.

## Required phone check — pending

Use the published revision in Bluefy, with motor power off. Do not ARM.

1. Connect, open **Connection, network & diagnostics**, and check mode/address/pending information.
2. Tap **Use car network**. Check confirmation and automatic Bluetooth disconnect. Reconnect and check the reported Car network state.
3. Tap **Use Home Wi-Fi**. Check confirmation and disconnect. Reconnect after the home attempt and inspect reported mode/pending/reason/address. BLE ownership pauses home Wi-Fi, so the address can be unavailable while connected.
4. Disconnect normally. Record the published build, phone/browser, visible outcomes and serial mode events when available.

No real iPhone/Bluefy result is claimed yet. Keep task 3.2 open and do not archive this change until it passes. Physical range fallback belongs to the home-network change and remains explicitly deferred.
