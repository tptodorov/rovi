# Five-minute motor endurance at the 70% cap

**Repeat physically confirmed; qualitative heating check passed.** The
[authorized repeat](2026-10-19-motor-endurance-repeat/notes.md) completed a
300,093 ms full-forward-to-zero interval, retained one UDP session and
restored normal ROS firmware and prior Wi-Fi. Todor confirmed continuous
motion of all four wheels, the final stop and still wheels through both
flashes/boots. He reported no warming in either driver boards or motors;
this is a qualitative observation without numeric temperature measurements. The [first authorized attempt](2026-10-19-motor-endurance/notes.md)
flashed the bench image but aborted at the unarmed announcement gate before
the probe launched. Normal ROS firmware and prior Wi-Fi were restored; Todor
confirmed all wheels stayed still through both flashes/boots and cleanup.
The controller allows a ten-minute unarmed announcement window. The repeat
establishes five-minute motor/radio endurance and a qualitative supervised
heating result on the stand. Coast/STBY pin-level measurements remain open
in task 5.2.

Use the same opt-in `ros,motor-bench` image and 70% product profile as the
[physically confirmed reversal repeat](2026-10-19-motor-reversals-repeat/notes.md).
No firmware change or rebuild is needed. The fixed UDP `vx=0.1` marker requests
`[1.0; 4]` from the product motor module, with the normal lease, STOP, release
and fault paths. Normal firmware must be restored afterwards. The bench's
serial demands are commands, not measured speed, duty, current or temperature.

## Announced batch

Wait for a separate go covering both flashes and this exact sequence. Allow
about seven minutes total. Rover restrained on its stand, all wheels clear,
motor bank awake, USB connected. Todor continuously supervises the entire run
and can remove motor power to abort. No stall or loaded test is authorized.

1. Verify the saved bench and normal ROS ELF hashes against the recorded
   reversal evidence. Flash the bench image, start a fresh serial capture,
   check its explicit bench banner/clean boot and join Rovi's AP. No router or
   firewall rule is needed.
2. Keep the board unclaimed/unarmed while waiting for the assistant to announce
   motion (controller window up to ten minutes). Expiry aborts and restores
   normal firmware; it never starts a drive. After the announcement, run:

   ```sh
   ROVI_TARGET=192.168.4.1:7777 python3 firmware/bench/motor_reversals.py \
     --carlog <fresh-live-car.log> --endurance
   ```

   Five-second lead-in, about one second armed at zero, then **all four wheels
   forward continuously for 300 seconds at the configured 70% cap**. There are
   no reversals or pauses. The 300-second clock starts only after fresh serial
   full-scale evidence, so discovery/delivery delay cannot shorten the hold.
   Progress prints every 30 seconds. Finally zero for about one second,
   STOP/BYE and fresh serial release.
3. The probe checks the original UDP session with repeated HELLO ACKs. A fresh
   initial full-scale serial state is required; subsequent unchanged state
   need not be logged repeatedly. Any reset, panic, fault, dropped event,
   stop/release, lost session or unexpected motor demand aborts. No recovery
   ARM is sent. Failure cleanup sends STOP/BYE and retries release; the normal
   one-second command lease also applies. Use an outer probe timeout of 330 s
   and unconditional restoration, rather than the reversal runner's 45 s.
4. Watch that all wheels keep turning forward for the whole hold. Unexpected
   slowing/stopping, stutter, unusual noise or smell is an abort, even if the
   serial checks still pass. Remove motor power if needed; do not continue or
   automatically retry. Record the physical reason. There is no temperature
   telemetry or temperature-based cutoff in this setup.
5. Stop capture, flash the exact recorded normal ROS image, then capture its
   clean boot with no bench banner or claim/arming for five seconds. Restore
   prior Wi-Fi and stop capture, including after an aborted run. Watch for
   twitch during both flashes and boots.

## Evidence and limits

Save source/image identities, flash logs, serial captures, probe output,
network cleanup and Todor's observations in dated notes. A pass requires
software checks plus continuous physical forward motion for the full hold,
the final stop, and still wheels during flashes/boots. A motor-power dropout
can leave the ESP32 and its command log healthy; physical observation matters.

Use the result label **visual stand endurance**, with any qualitative heating
observations attributed to the observer. No warming was reported in the
confirmed repeat; no numeric temperature bounds, thermal-shutdown margin,
current safety or loaded operation were measured. This procedure does not
change the product requirement or close task 5.2.

The accelerated real-socket test verifies a continuous forward hold with only
one full-scale state event and the elapsed hold interval despite delayed
first-marker delivery. A mid-hold STOP
negative control must fail and release ownership. Existing reversal, normal
image refusal, log-ordering and cleanup retry tests remain in CI discovery.
Simulation provides no electrical or temperature evidence.
