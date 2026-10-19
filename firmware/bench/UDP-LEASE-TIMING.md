# UDP lease timing at zero wheels

Board runs require the separately announced batch and Todor's go. Use the
installed product firmware; no flash. Rover on its stand, wheels clear, USB
connected and motor bank awake. Capture serial before joining Rovi's AP.

```sh
ROVI_TARGET=192.168.4.1:7777 python3 firmware/bench/udp_lease_timing.py --carlog <live-car.log>
```

Five trials, each about one second:

1. Claim over UDP and read the advertised lease length.
2. Record the laptop's monotonic clock immediately before sending a **single
   zero setpoint**, then ARM. Successful ARM proves the setpoint was accepted.
3. Send no more commands. Timestamp receipt of the fresh serial `LeaseExpired`
   event using the same laptop clock. Require Armed state, zero wheel commands,
   no reset/fault/dropped events, and the unsolicited lease-expired UDP Status.
4. Require the elapsed time to be at most the advertised lease + 50 ms
   (1,050 ms for the installed profile). STOP/BYE and observe release, including
   after assertion failure, before another trial.

This is a conservative upper bound: it includes outbound UDP delivery, serial
capture, file polling and laptop scheduling. It requires no synchronization
with the ESP32 clock. A pass establishes the lease-detection bound for that
trial; a failure may come from either the board or delivery/scheduling delay
and needs investigation. No percentile or average substitutes for a failing
trial. Serial timestamps are retained as evidence but are not subtracted from
laptop timestamps. The probe does not measure STBY propagation or wheel
stopping time, and covers UDP timing only; BLE/ROS timing remains pending.

No nonzero command is sent. Watch that all four wheels stay still. Restore the
previous Wi-Fi connection and stop capture afterwards. Save probe output,
serial log, firmware identity, network cleanup and observation in dated bench
notes; do not mark physical acceptance from simulator results.

`test_udp_lease_timing.py` exercises the complete probe on the product simulator
and verifies that delayed serial observation fails the bound while releasing
ownership. The existing CI bench-test discovery includes it.
