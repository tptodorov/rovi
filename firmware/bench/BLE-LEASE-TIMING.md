# BLE lease timing at zero wheels

Run only after the separately announced batch and Todor's go. **The current
probe requires the instrumented product firmware**, which records
`lease_late_ms` on lease stops. Flash that build only as part of an authorized
batch. Rover on stand, wheels clear, USB connected and motor bank awake. Start
serial capture before the probe. No laptop Wi-Fi or firewall change is needed;
the car disables its AP while BLE owns it.

```sh
nix-shell --run 'python3 firmware/bench/ble_lease_timing.py --carlog <live-car.log>'
```

One BLE connection, five trials, about 30 seconds including setup/cleanup:

1. Read capabilities and require the installed 1,000 ms lease, a sufficient
   ATT MTU, and Claimed state without errors.
2. Timestamp immediately before sending one zero setpoint, then explicitly ARM.
   Successful ARM proves acceptance. Send nothing else until the lease expires.
3. Timestamp observation of the fresh serial `LeaseExpired` event. Require a
   fresh Armed state, all wheel commands zero, no reset/fault/drop/release,
   and a BLE lease-expired Status notification from this trial.
4. Require an integer **`lease_late_ms` from 0 through 50** in that stop event.
   Missing, negative, malformed or greater values fail. Repeat with a new zero
   setpoint and explicit ARM; ownership remains on the same connection.
5. STOP and disconnect, and require a fresh serial release before reporting
   success. Cleanup also runs after an assertion or failed STOP.

The device computes `lease_late_ms` at the stop decision from its actual lease
deadline, before clearing it. That deadline was set when the sole valid zero
setpoint was accepted. The duration uses one device clock, independently of
event-queue, BLE, serial or laptop delivery delays. The requirement remains
lease + 50 ms, equivalent to at most 50 ms after the deadline.

The laptop also records `host_upper_bound_ms` from before send to serial
observation using its own monotonic clock. This includes BLE delivery, serial
capture, polling and laptop scheduling; it is diagnostic and is printed even
when it exceeds 1,050 ms. A prompt device stop with a larger host bound points
to delay outside the stop decision. It cannot distinguish BLE delay from
serial/laptop delay. Do not mix ESP32 and laptop timestamps or average away a
failing device trial. Notification receipt is checked separately after
recording serial observation time.

The [first board attempt](2026-10-19-ble-lease-timing/notes.md) used the original
host-bound-only probe at `270716a`. Its 1,051.064 ms result remains a failed
probe with an unresolved firmware timing verdict; the recorded Armed-to-stop
interval cannot supply the missing deadline. It is not reclassified as a pass
by this new measurement method. Installed older firmware must be replaced
before another attempt; the new probe fails if timing evidence is absent.

All four wheels must stay still; Todor's observation is recorded separately.
Save the source/firmware identity, probe output, serial capture and cleanup in
dated bench notes. This does not measure STBY propagation or physical stopping
time. UDP timing has separate evidence; ROS timing remains pending.

`test_ble_lease_timing.py` checks five zero commands, prompt device stops with
slow host observation, the 50/51 ms boundary, missing/malformed timing,
stale/missing evidence, wrong stop reason, nonzero wheels, MTU rejection and
cleanup failures with a fake client and clock. Core tests verify the deadline
source, effect merging, delayed logging and motor-fault replacement. CI already
discovers these tests. They establish measurement safeguards only; neither a
fake client nor the simulator proves BLE timing on the board.
