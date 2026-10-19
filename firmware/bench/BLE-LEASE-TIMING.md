# BLE lease timing at zero wheels

Run only after the separately announced batch and Todor's go. Use the installed
product firmware; no flash. Rover on stand, wheels clear, USB connected and
motor bank awake. Start serial capture before the probe. No laptop Wi-Fi or
firewall change is needed; the car disables its AP while BLE owns it.

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
4. Require the observed upper bound to be at most 1,050 ms. Repeat with a new
   zero setpoint and explicit ARM; ownership remains on the same connection.
5. STOP and disconnect, and require a fresh serial release before reporting
   success. Cleanup also runs after an assertion or failed STOP.

Both timestamps use the laptop's monotonic clock. The upper bound includes
BLE delivery, serial capture, polling and laptop scheduling. A passing trial
establishes the lease-detection bound; a failure may also reflect delivery or
scheduling delay and needs investigation. Do not average away a failing trial
or mix ESP32 and laptop timestamps. Notification arrival is checked separately,
after recording the serial observation time.

All four wheels must stay still; Todor's observation is recorded separately.
Save the source/firmware identity, probe output, serial capture and cleanup in
dated bench notes. This does not measure STBY propagation or physical stopping
time. UDP timing has separate evidence; ROS timing remains pending.

`test_ble_lease_timing.py` checks the five zero commands, late/stale/missing
evidence, wrong stop reason, nonzero wheels, MTU rejection and cleanup failures
with a fake client and clock. CI already discovers these tests. They establish
probe safeguards only; neither a fake client nor the simulator proves BLE
radio timing.
