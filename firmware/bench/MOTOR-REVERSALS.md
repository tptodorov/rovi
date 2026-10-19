# Product motor-module reversal batch

**Physical reversal check passed.** The [separately authorized repeat](2026-10-19-motor-reversals-repeat/notes.md)
completed ten direct all-wheel reversals at the configured 70% cap. Todor
confirmed directions, the final stop and no twitch through both flashes/boots.
Normal ROS firmware and prior Wi-Fi were restored. The [first attempt](2026-10-19-motor-reversals/notes.md)
remains an abort at zero caused by a host log-prefix checker bug, reproduced
and fixed before the repeat. Coast/STBY measurements need a meter/probe
(currently unavailable); thermal qualification remains a separate supervised
batch, so task 5.2 stays open.

The same image also supports a separately authorized [five-minute visual
endurance batch](MOTOR-ENDURANCE.md) with `--endurance`. It has no temperature
measurement and cannot close thermal acceptance.

The ordinary car API cannot request all four wheel commands at ±1.0 together
within its existing limits. The opt-in `motor-bench` image instead supplies
fixed vectors to the **same product motor module and safety coordinator**:

```text
UDP vx=+0.1 marker → FL/FR/RR/RL = +1.0 → 70% forward
UDP vx=−0.1 marker → FL/FR/RR/RL = −1.0 → 70% reverse
zero/other demand → zero wheel commands (coast while armed)
STOP / expired lease / release / motor fault → normal standby path
```

Markers are the CAR profile's four equal ±83-permille normalised demands.
Only UDP ownership can select the fixed vectors; BLE/ROS demand coasts in this
image. There are no API, profile or geometry changes in normal builds. The
bench feature is absent by default. It prints `BENCH ONLY motor-reversal ...`;
`ev=state wheels_pm` records the injected motor demands, including ±1000,
rather than the ordinary kinematic demands. Those are requests, not measured
wheel speed or electrical duty.

## Build and preflight

Build with `ros,motor-bench` in `target/reversal-bench` using the existing AP
credential in the build environment. The separate output preserves the normal
ROS image in `target/ros`. Firmware commands require the repository Nix shell
and ESP toolchain (see the root README). Record source revision and ELF hashes.
Check the normal image's hash against its recorded identity before using it for
restoration; do not overwrite it with the bench build.

**Announce the two flashes and exact motor sequence, then wait for Todor's go.**
Rover restrained on its stand, all wheels clear, motor bank awake and USB
connected. This is full 70% drive duty, substantially faster than earlier
gentle probes. Todor watches throughout and can remove motor power to abort.

After authorization:

1. Flash the bench ROS image. It starts unarmed; flashing/boot must not move a
   wheel. Start a **fresh** serial capture, check its bench banner and clean
   boot, then join Rovi's AP. No router or firewall rule is needed.
2. Run:

   ```sh
   ROVI_TARGET=192.168.4.1:7777 python3 firmware/bench/motor_reversals.py --carlog <live-car.log>
   ```

   The probe waits 5 s, claims one UDP session and arms at zero for 1 s. It
   drives forward for 1 s, then alternates reverse/forward every second for
   **ten direct reversals** (11 full-duty holds, ending forward). There is no
   zero or standby interval between reversals. It then commands zero for 1 s,
   sends STOP/BYE and requires fresh serial release.
3. Each hold requires fresh exact motor demands and repeated HELLO ACKs in the
   original session. Reset, panic, stop, release, fault, dropped events,
   mismatched/missing demand or lost radio/session fails the batch. Failure
   cleanup sends STOP/BYE and retries release; a lost laptop stream is also
   subject to the ordinary one-second lease. Do not continue after a failure.
4. After the probe ends (including a failed attempt), stop capture and restore
   the recorded **normal ROS firmware**, then capture its clean boot. Verify
   the bench banner is absent and the car remains Open with zero commands and
   no new claim/arming for 5 s. Restore the prior Wi-Fi and stop capture.

## Evidence and acceptance

Save probe output, fresh serial logs for both images, build identities,
cleanup/network results and Todor's observation in dated notes. Physical pass
requires all four wheels following each announced direction, no reset/session
loss, stopping after the last hold, and no twitch through either flash/boot.
An abort or failure stays recorded as such. This short run does not qualify
temperature, stall current, loaded operation or coast pin levels.

Development checks exercise the real coordinator with a recording pin driver:
all four bridges receive 700 permille in either direction, with no standby
between reversals. They also check zero/unsupported demands, the unchanged
normal path, STOP, lease expiry, release and motor faults. A separate opt-in
simcar runs the full socket probe; negative controls reject a normal image
before claim and wrong motor evidence while still releasing ownership. Dropping
the first cleanup BYE verifies the retry. CI builds the bench ROS firmware and
runs these checks. Simulator pins do nothing; hardware acceptance is pending.
