# Wi-Fi stall observation

First authorized baseline completed 2026-10-20: [4,385 Echo replies, zero loss](2026-10-20-wifi-soak/notes.md). #23 remains open; the original ROS workload was not replayed. Each future run needs its own announced go.

Investigates [#23](https://github.com/tptodorov/rovi/issues/23): the laptop
remained associated with Rovi M8 while all traffic stopped after roughly ten
minutes. Cause is unknown. A clean observation does not close the issue.

The recorder sends only stateless UDP Echo frames with session zero. It never
claims, arms, sends setpoints, changes power saving or reconnects Wi-Fi. Its
read-only snapshots include three ICMP pings. No runtime dependency is added.

## Announced board batch

About 16 minutes. A separate go is required before joining the board network or
opening serial capture. USB connected; motor power disconnected. Keep the
installed normal ROS firmware; no flash, BLE connection, router or firewall rule.

1. Save the current Wi-Fi UUID and internet route. Start fresh serial capture
   (one board reset), verify a normal boot with no claim or product fault, then
   join the existing `rovi-m8` NetworkManager profile.
2. Run the recorder for 15 minutes, leaving the connection unchanged even if it
   stalls. Do not start the teleoperation client or another controller.
3. After the recorder has saved diagnostics, if Echo remains unreachable,
   rejoin `rovi-m8` once and run a separate ten-second observation to record
   whether reachability returns. No repeat of the full soak is automatic.
4. Stop serial capture and restore the previous Wi-Fi, even on failure or
   interruption. Check the internet route. Audit serial for any unexpected
   claim, arming, nonzero wheels, reboot, fault, dropped event or panic.

From the repository root, after the authorized setup:

```sh
python3 firmware/bench/wifi_soak.py --target 192.168.4.1:7777 \
  --interface wlp1s0f0 --seconds 900 --output /tmp/rovi-wifi-soak
```

The output directory must be new. Exit 0 means no lost Echo observed; 1 means
some loss; 130 means interrupted; 2 reports setup/I/O failure. One lost packet
is not by itself a reproduction of the sustained stall.

## Evidence and limits

* `echo.jsonl`: UTC/monotonic timing, sequence, matched reply/timeout and RTT.
  Only correct endpoint, framing, session, sequence and nonce count as success.
* `start.json`, `transition-NNN.json`, `end.json`: NetworkManager connection and
  IPv4 state, `iw` association and power-save state, route to the target and
  ICMP results. Each command records its exit/output or missing-tool/timeout
  error. Snapshots pause probing; their start/end and sample timestamps show
  these gaps. At most five probes/s, not a timing guarantee.
* `journal.json`: up to the last 2,000 NetworkManager/wpa_supplicant journal
  lines since setup began. Permission errors and unavailable units remain
  visible; no debug logging or privilege changes are made.
* `summary.json`: attempted/lost counts and complete/incomplete verdict. The
  recorder reports that it requested no reassociation; the OS might reconnect
  independently, which needs checking in the journal and snapshots.

Correlate persistent Echo loss with ICMP, association, power saving and serial
before assigning a cause. This first baseline has no ROS router/keepalives;
it does not recreate the original traffic exactly. Probes may affect inactivity
behavior. Neither a negative run nor a simulator pass establishes a fix.
Review network/service logs before publishing them; preserve raw evidence and
document any redactions. Host tests run in CI through the existing bench suite.
