# 2026-10-19: BLE lease timing — first-trial upper bound exceeded

Todor authorized the announced five-trial batch with “go”. Rover on stand,
wheels clear, USB connected and motor bank awake. Installed ROS product
firmware from the [read-only batch](../2026-10-18-ros-readonly/notes.md);
no flash. Probe source `270716a` (PR #81), following the
[procedure](../BLE-LEASE-TIMING.md).

**Result: probe failed; BLE lease-detection bound remains unverified.** The
first trial's conservative laptop-send-to-serial-observation upper bound was
**1,051.064 ms**, above the **1,050 ms** limit for the advertised 1,000 ms lease.
The probe raised its assertion and ran cleanup. Trials 2–5 did not run. This
result is retained; no retry or tolerance change was made.

| Evidence | Observation |
| --- | --- |
| [probe.txt](probe.txt) | Assertion failure on the first trial; `rc=1` |
| [car.log](car.log) | Armed at 5,360 ms, all four wheel commands zero; LeaseExpired at 6,360 ms; cleanup STOP at 6,393 ms; release/BLE down at 8,822 ms; Open at 8,828 ms; AP on at 8,844 ms |
| [network.txt](network.txt) | Previous laptop Wi-Fi and Ethernet internet route unchanged; no firewall change |
| Physical observation | Todor confirmed: “Yes, all four stayed still” during the trial and cleanup |

The 1,000 ms Armed-to-stop interval is diagnostic only: the zero setpoint was
accepted before ARM, and its exact acceptance time is absent from this log.
It does not establish the setpoint-to-stop deadline. Conversely, the failed
conservative upper bound includes outbound BLE, serial capture, polling and
laptop scheduling; exceeding it does not identify firmware detection as late.
The failure occurred before the separate notification assertion, so this run
does not claim a verified lease Status notification either.

Audit: one boot, one zero-wheel Armed state, one lease stop and one release;
every wheel command zero; no errors, dropped events or panics. Capture stopped.
No firmware changes or flash. UDP timing retains its separate passing evidence;
BLE/ROS timing, coast/STBY levels and reversal/thermal checks remain pending.

Next measurement needs enough timing evidence to separate setpoint acceptance,
firmware stop detection and host/transport delay. This capture cannot localize
the excess 1.064 ms; any instrumented board repeat must be announced separately
and authorized before running. No numeric STBY propagation or physical stopping
time is inferred.
