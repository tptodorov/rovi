# Physical input in the existing iPhone client

## Boundary

Keep one Session owner and one serialized BLE stream. A small Physical interpreter
beside Hold supplies bounded intent; it never arms, connects or writes BLE. Keep
button Hold behavior intact. All Session invalidation paths clear both holds;
close also disables physical input and discards calibration.

## Input and permissions

Default Buttons. Selecting Physical does not request permission or claim a car.
Enable Sensors calls the optional DeviceOrientationEvent permission method directly
from the tap, then installs generation-guarded orientation callbacks. No compass,
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
end Physical sessions. Do not silently recenter or rotate axes under an active hold.

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
post-arm hold are mandatory. A ready connection can remain open after confirmed
STOP. Permission alone never arms. Connecting discards earlier calibration;
ARM completion clears holds but preserves valid calibration.

Physical nonzero requires active foreground, local arm, calibration, one tracked
drive-pad pointer and a fresh orientation sample. UI frames alone renew the hold
(250 ms). Only new valid callbacks renew sample freshness (250 ms). Test expiry
before replacing an old sample, so a late callback cannot hide a missed deadline.
Cached values and transmitter ticks cannot renew either input deadline.

Release/bounds exit/cancel/multi-touch/scroll clears the hold before another send,
with new press required. Sensor/posture loss during a held drive or hold expiry
revokes calibration and ends through cleanup. When released, a sample gap or
out-of-envelope reading posture blocks new holds at zero but retains neutral and
arm. Rejected touches stay blocked until release; fresh data cannot replay them.
Invalid values or changed reference still revoke calibration and close armed or
arming sessions. Return after lifecycle loss requires manual sensor enabling,
connection, Set neutral & ARM and a new hold.
Old permission completions and detached callbacks cannot affect a later mode/session.

Keep a sticky safety panel outside the drive section: local arm state, ARM/STOP
and the latest session reason stay visible through scrolling and fault cleanup.
Show temporary sensor/posture unavailability near the physical pad. The phone's
original scroll trigger was not captured; host tests reproduce released-pad gaps
and reading posture without weakening stream or lifecycle deadlines.

Keep 50 ms attempted sends, 200 ms scheduler/submission failure, Status validation,
STOP priority and 500 ms runnable cleanup unchanged. Phone measurements found a
60 ms median for a nominal 50 ms local consumer: record actual BLE gaps rather
than claiming exact 20 Hz. Firmware lease remains an independent fallback.

## Evidence and remaining gates

Pure input tests plus Session fake-GATT and real-page browser tests cover this
state model; they do not establish phone/BLE or physical stopping. Phone acceptance
must combine sensors and the full BLE workload, exercise all invalidations and
extend quiet evidence to 60 s. A separate stand batch covers six signs, mixed axes,
neutral/release, STOP, mode changes and interruptions. No floor test is included.
