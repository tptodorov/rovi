# 2026-10-21: iPhone three-axis sensor feasibility

Todor ran the isolated HTTPS diagnostic at revision `ce853f16ef7d` in Bluefy.
No rover connection, BLE commands, firmware change or motor run was involved.
The [extracted report](report-excerpt.json) preserves selected complete sections
of his pasted report; the paste stops mid-event and is not a complete JSON export.
No missing tail has been reconstructed.

## Scope and result

**Pass for sensor availability and the reported local three-axis interaction.**
Proceed with the selected Buttons / Physical input design and host implementation.
This is not sensor-plus-BLE, motor or long-term drift acceptance.

The user agent reports iOS 18.7 and Bluefy 3.9.3. Manual version and observation
fields are empty; this report does not identify the phone hardware model.
Both orientation and motion permissions were granted in a secure context.
All 14,437 orientation readings had finite alpha/beta/gamma; all 14,438 motion
readings had finite acceleration, gravity and rotation-rate axes.

| Arrival stream | Recent median | Recent p95 | Whole-run maximum gap |
| --- | --- | --- | --- |
| Orientation | 17 ms | 22 ms | 14,998 ms |
| Motion | 17 ms | 22 ms | 14,984 ms |
| UI frames | 33 ms | 40 ms | 15,765 ms |
| Local consumer, scheduled at 50 ms | 60 ms | 61 ms | 15,034 ms |

Counts and maxima span enabled segments and interruptions; percentiles use the
latest 4,096 gaps. These are callback-arrival measurements, not acquisition or
end-to-end control latency. The scheduled 50 ms consumer did not establish an
actual 20 Hz rate on this host. Sensor-plus-BLE timing still needs its own check.

Todor confirmed:

- All three previews matched intended forward/back, strafe and rotation directions,
  stayed steady while holding a twist, and returned to zero at neutral or release.
  No reversed sign, unwanted axis motion or heading jump was reported.
- “Orientation expired; recalibrate” occurred **only during deliberate pauses or
  interruptions**, not ordinary foreground use.

The event prefix records expiry, blur/hidden, a later Enable and new calibration.
The truncated tail prevents assigning every large gap to a specific test or
auditing each stall/lock/orientation scenario separately. User confirmation
supports the observed interaction; it does not measure a stopping deadline.

## Quiet readings and remaining limits

The first saved neutral trial lasted **13.126 s**, with 788 readings. Peak-to-peak
variation was about 0.043° yaw, 0.070° pitch and 0.052° roll; endpoint offsets were
−0.010°, +0.027° and −0.002°. This supports trying the existing 3° dead zone for
this posture. It does not establish 60 s or long-term drift behavior.

The second saved trial lasted **16.228 s**, with 972 readings and excursions up
to roughly 26° yaw, 15° pitch and 23° roll. It contains substantial movement and
cannot be treated as stationary sensor noise. Neither saved trial is a full
quiet minute. Mixed-axis behavior is user-confirmed, not a measured coupling bound.

The raw `motion.interval` is `0.01666666753590107`, whereas measured arrivals are
about 17 ms. The [W3C contract](https://www.w3.org/TR/orientation-event/#dom-devicemotionevent-interval)
defines this field in milliseconds; its value here disagrees with observed cadence.
Use monotonic callback arrival times for freshness and scheduling, without guessing
a conversion or integrating gyroscope rates. Compass accuracy includes −1; no
reliable compass or heading-source guarantee is inferred. Relative neutral offsets
remain the selected interaction.

Pending product checks: explicit mode switching, ARM/STOP races, independent
sensor/UI expiry, permission loss and manual recovery; sustained sensor-plus-BLE
stream timing; a separately announced/authorized stand batch. A fresh 60 s neutral
sample can extend the drift evidence during that phone acceptance.
