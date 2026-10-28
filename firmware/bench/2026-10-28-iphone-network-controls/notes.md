# iPhone network controls — Bluefy check

**Passed for the tested phone setup, 2026-10-28.** Served client build `82fbd45f4050`, linked to Todor for Bluefy testing. The page, session and transport assets were checked against the local tested build. The phone model and Bluefy version were not recorded; wider-device qualification is not claimed.

Todor reported performing the network steps several times and confirmed the final fresh display after reconnect: **Home Wi-Fi · Pending: none** (“correct”). He then confirmed normal disconnect (“yes, disconnected fne”). The retained serial capture independently shows the requests, ownership release, mode changes and reconnection.

| Sequence | Serial evidence |
| --- | --- |
| Home while at home | `mode_cmd source=Ble cmd=Home result=Ok` at 278746 ms; release/disconnect at 279045 ms; Home at 280107 ms |
| Home → Car network | CarNetwork accepted at 296476 ms; release at 296775 ms; Fallback/Requested at 296788 ms; software reset boots into Fallback/Requested |
| Car network → Home | Home accepted at 131758 ms in the next boot; release at 132088 ms; software reset; DNS resolves blackbox and Home is reached at 2188 ms in the new boot |
| Fresh observation | BLE reconnect at 240089 ms and disarmed STOP claim at 242163 ms; Todor confirms Home Wi-Fi/Pending none on the page |
| Cleanup | Todor confirms normal disconnect; capture shows release and subsequent Home recovery; monitor stopped and serial released |

The expected software resets are part of the existing mode-switch implementation. Millisecond counters restart at each boot. Network requests occurred after a STOP; the network-command phase contains no Armed state or fault event. The capture also includes earlier user-operated motion, which Todor described as working. It is separate from this network check and does not establish a new motor-acceptance batch.

## Observations and limits

The initial monitor command with `--no-reset` stalled before usable capture. It was stopped and replaced with the previously working monitor command, intentionally rebooting the board once. The recovered capture initially shows a brief BLE connection/drop before a successful connection. Todor then reported that connecting and motors worked. No root cause for the initial disappearance is established, and no firmware/client fix is claimed for it.

Todor needed help locating the expandable **Connection, network & diagnostics** section. After a Home request, the disconnected page correctly retained **Last reported: Car network · Pending: Home Wi-Fi**; reconnecting replaced it with fresh Home/Pending none. The retained display is not a live join-progress indicator.

No phone screenshot or exported client report was captured. Physical motor-power state during network actions was not independently recorded; serial verifies the disarmed network-command phase. Automated refusal, delayed GATT, malformed/legacy metadata and stale-callback cases remain host evidence, not injected phone faults.

[car.log](car.log) retains the full recovered session. [identity.json](identity.json) and [capture-end.json](capture-end.json) identify capture setup/cleanup; [manifest.json](manifest.json) records raw and published SHA-256. SSID and MAC addresses are redacted; the home passphrase is absent. Physical range fallback remains explicitly unverified in the separate home-network change.
