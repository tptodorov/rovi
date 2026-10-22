# Physical input in the existing iPhone client

## Boundary

Keep one Session owner and one serialized BLE stream. A small Physical interpreter
beside Hold supplies bounded intent; it never arms, connects or writes BLE. Keep
button Hold behavior intact. All Session invalidation paths clear button holds and Physical running state;
close also disables physical input and discards calibration.

## Input and permissions

Default Buttons. Selecting Physical does not request permission or claim a car.
Selecting Physical or an explicit Retry calls the optional DeviceOrientationEvent
permission method directly from that gesture, then installs generation-guarded
orientation callbacks. No compass,
motion.interval, gyroscope integration or sensor fusion is needed. Missing/null/
non-finite/out-of-range angles fail closed. Track optional absolute-reference
metadata; a change invalidates calibration rather than silently changing the frame.

Set neutral & ARM takes the latest fresh sample while locally disarmed with no
pending control, then follows the existing zero/ARM/read sequence. No separate
calibration tap is required; every explicit ARM captures neutral, never a sensor
callback. Neutral must be screen-up, beta/gamma within ±30°. Current beta/gamma
must remain within ±60°, and shortest heading offset within ±90°. These explicit
software limits keep the selected Euler mapping away from inversion/singular
postures; they are not measured sensor-accuracy limits. Screen orientation changes
end Physical sessions. Do not silently recenter or rotate axes under running Physical control.

## Mapping

Use shortest signed angle differences, including alpha wrap at 0/360. Map
negative delta-beta to +vx, negative delta-gamma to +vy and positive delta-alpha
to +wz. Each component has a symmetric 3° dead zone, linear scale to full value
at 20°, then saturation: min(0.1 m/s, advertised translation maximum), or
min(0.3 rad/s, advertised rotational maximum). Zero axes stay zero. All three
components may combine in Physical mode. Buttons continue to select one axis.
No filtering initially: the user confirmed this preview, with quiet excursions
below 0.05° for 13.1 s. Longer drift and mixed-motion physical bounds remain unverified.

## State and recovery

Mode changes clear both inputs and calibration, revoke local arm, supersede ARM
with STOP while ready, and cancel setup/picker attempts. A fresh ARM and new
post-arm input action are mandatory. A ready connection can remain open after confirmed
STOP. Permission alone never arms. Connecting discards earlier calibration;
ARM completion clears input but preserves valid calibration.

Physical mode uses explicit tap START/PAUSE after confirmed Set neutral & ARM.
START requires all three shortest offsets within ±3° of neutral, fresh valid
orientation and active foreground; an early/rejected START is never queued.
START latches running input, independent of finger release. PAUSE and scroll clear
running input to zero while retaining arm/neutral; no sample can restart it.
STOP, mode changes, ARM completion, cleanup and reported safety stops also clear
running state. Buttons retain their existing held-pointer interpreter.

Only UI frames renew the Physical running deadline (250 ms); only new sensor
callbacks renew sample freshness (250 ms). Test expiry before replacing samples
or renewing a deadline, so resumed execution cannot hide a missed deadline.
Sensor/posture loss while running or foreground-renewal expiry revokes calibration
and ends through cleanup. When paused, gaps or reading posture block START at zero
but retain neutral and arm. Invalid values/reference changes still close armed or
arming sessions. Lifecycle return requires manual enabling, connection, ARM and
START. No silent recentering or automatic recovery is introduced.

Keep requested motion and the applicable ARM/START/PAUSE action with STOP in the
persistent thumb dock; arm state and the latest reason remain in a sticky status
area. Give running/paused text and the offending
angle/value/limit on posture faults. Screen-up tilt limits stay ±60° absolute,
heading offset ±90°; the interface change does not establish a cause for the
reported z-axis interruption. Earlier held-pad UX/neutral results are evidence for
that build, not acceptance of this new interaction.

Keep 50 ms attempted sends, 200 ms scheduler/submission failure, Status validation,
STOP priority and 500 ms runnable cleanup unchanged. Phone measurements found a
60 ms median for a nominal 50 ms local consumer: record actual BLE gaps rather
than claiming exact 20 Hz. Firmware lease remains an independent fallback.

## Evidence and remaining gates

Pure input tests plus Session fake-GATT and real-page browser tests cover this
state model; they do not establish phone/BLE or physical stopping. Phone acceptance
must combine sensors and the full BLE workload, exercise all invalidations and
extend quiet evidence to 60 s. A separate stand batch covers six signs, mixed axes,
neutral/PAUSE, STOP, mode changes and interruptions. No floor test is included.


## Controller screen redesign

Todor requested a production-quality interface for both modes using design skills.
Apply the local UX Designer and Web Interface Guidelines skills within this
existing client, with no framework, remote font, runtime asset or hosting change.
Keep Session, Hold, Physical, wire values and motor limits as the control boundary.

Portrait wireframe (compact, 320–390 px):

```text
rovi / controller       connection · Connect/Disconnect
[       Buttons       |       Physical       ]
Phone arm state · next action / inline fault reason
[ six held directions OR live tilt/neutral visual ]
Requested: forward · sideways · turn
[ Connection & diagnostics (closed) ]
[ ARM / START / PAUSE              ][ STOP ]  thumb dock
```

Use system typography, warm neutral surfaces, forest-green identity, amber for
armed/paused and red for STOP/faults. Support a matching dark palette. Use SVG
icons/phone input visualization with text equivalents, tabular numerical values,
44 px minimum targets and visible keyboard focus. Default and connected phone
screens must show controls and requested motion without scrolling; larger text
may scroll with safe bottom padding so focus stays above the dock. Keep actual
fault messages visible. Show technical Status, stream counters and raw angles in
collapsed native details; opening it pauses input. Never animate motion merely
for decoration or imply measured wheel feedback.

Choosing Physical from a user-operated native radio prepares orientation sensors
in that same gesture; it never connects or arms. Allow this while a mode-change
STOP is pending because local motion is already revoked. Permission denial or
missing readings expose Retry; the normal path needs no separate Enable button.
Retain epoch/generation/foreground guards and actual blur/hidden cleanup, including
if an iOS permission dialog interrupts an existing connection. Native permission
and Web Bluetooth may still require manual recovery on the phone; no permission
exception weakens safety or promises a production qualification we have not tested.

Render text only when it changes, so a stable status is not re-announced every
frame. Keep sensor previews separate from requested velocities; paused previews
never authorize motion. Test compact/light/dark/200% text states plus permission,
mode, input cancellation, STOP, diagnostics and lifecycle paths. This redraw needs
a new motor-power-off Bluefy usability check; earlier phone results are historical.
