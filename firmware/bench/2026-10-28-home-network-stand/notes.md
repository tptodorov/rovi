# Home network mode: stand acceptance 6.2, 2026-10-28

Todor authorized continuing 6.2, confirmed “Stand ready; I can observe”, and after the motor batch confirmed “Yes, motion and stops looked correct.” The announced batch used the normal home-mode ROS image with `tcp/blackbox.local:7447`, USB serial capture, gentle forward requests (`vx=0.1 m/s`, wheel commands `83,83,83,83`) and explicit stop/release cleanup. ROS forward pulses expired automatically after one second; UDP/BLE write sequences lasted under one second. This is stand evidence, not floor or loaded qualification.

## Results

| Check | Evidence | Result |
| --- | --- | --- |
| ROS claims/auto-arms on the home LAN and drives forward | [takeovers](takeovers-car.log), [ROS](takeovers-ros.log) | One-second forward pulse, then zero |
| UDP takes over from moving ROS | same | ROS preempted with OwnerLost, UDP claims disarmed, wheel commands zero |
| UDP needs ARM | same | Nonzero requests before ARM stay zero; explicit ARM permits the gentle pulse; STOP zeros |
| UDP leaves without Home | same | ROS claims disarmed, nonzero requests stay zero until `/rovi/arm` true, then the pulse works |
| BLE observes while ROS owns, then takes over | same | Watcher reads Armed without claiming; first STOP preempts ROS and stops the wheels |
| BLE needs ARM | same | Pre-arm nonzero input stays zero; explicit ARM permits the gentle pulse |
| Deliberate BLE hand-back | same | Home pending before disconnect; rejoins home, ROS auto-arms and drives again |
| BLE disconnect without Home | [accidental-drop follow-up](car.log), [ROS](ros.log) | Safe stop; rejoin; ROS claims disarmed and its nonzero requests stay zero; explicit `/rovi/arm` true enables the next pulse |
| Mode command both ways, BLE | [mode capture](modes-car.log) | CarNetwork → fallback; Home → home |
| Mode command both ways, UDP | same | CarNetwork over home LAN → fallback; Home over car AP → home |
| Mode command both ways, ROS | same, [home node](modes-home-ros.log), [fallback node](modes-fallback-ros.log) | Disarm then mode + release on each network; fallback router temporarily on blade; all six mode commands accepted; all seven boots/transitions captured; no nonzero commands |
| Operator observation | Todor's reply in this session | Forward motion and stops, including takeovers, looked correct |

Every retained serial log passes `carlog.check`: no panic, product error, dropped log events or ownership ordering errors. [Analysis](analysis.json) records event counts, mode commands, nonzero states, stops and the flashed ELF SHA-256. The image was built from the router-by-name working changes on base `73cbef8`; the build's source modifications are committed with this evidence. Resolution verification is [separate](../2026-10-29-router-by-name/notes.md).

## Test-harness interruptions

The batch was completed across the recorded captures rather than counted as one uninterrupted run:

1. [Setup capture](setup-car.log), [ROS](setup-ros.log): the first ROS pulse and UDP takeover passed, then an assertion expected a repeated zero-state event. Firmware logs only state changes. The harness was corrected to verify the latest state plus absence of any subsequent nonzero state.
2. [Takeover capture](takeovers-car.log) passed the UDP and deliberate BLE sections. The final BLE disconnect assertion expected OwnerLost, but BlueZ disconnect took about two seconds. The one-second lease expired first (`LeaseExpired`, zero wheels); release and rejoin followed, and ROS remained disarmed. In that capture, the nonzero BLE state lasted 1447 ms until lease expiry, longer than the announced one-second pulse; no longer pulse was requested, but disconnect itself did not zero immediately. The follow-up checks either valid safety stop, then directly proves the latch and explicit re-arm. This does not claim an OwnerLost-first stop on physical RF loss.
3. [Initial mode capture](setup-modes-car.log): BLE mode commands both applied, but resetting the board closed BlueZ's disconnect operation with EOF. The harness now verifies the serial transition after that expected reset; the complete repeat passed all six mode commands.

Executed harness snapshots: [stand](stand.py.txt), [ROS node](ros.py.txt), [mode checks](modes.py.txt). They use the existing [6.1 bench helpers](../2026-10-26-home-network-mode/bench.py.txt) under `/tmp/rovi-61`; these are execution snapshots, not a new supported CLI. The completed captures precede the final cleanup-only simplification in the ROS snapshot (single stop/release instead of repeating zero/stop/release, which could briefly reclaim at zero); the command and motion phases are unchanged.

## Cleanup and scope

Normal named-router image retained; final network Home, ROS connected to blackbox, no owner and zero wheels. Serial monitor stopped and port free. The two temporary blade containers and the scoped firewall rule were removed. Original Wi-Fi restored; home route remains wired. Actual home passphrase absent from every retained log; SSID redacted.

**6.2 passes for the supervised stand checks above.** The 15-minute coexistence soak and out-of-range fallback remain 6.3. No loaded/stall, current, thermal, stopping-distance or wider-network qualification is claimed.
