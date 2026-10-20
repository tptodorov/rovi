# 2026-10-20: laptop teleoperation stand acceptance

Todor authorized the [announced batch](../LAPTOP-TELEOP.md) with “go” and
confirmed afterwards: **“Yes, all behaved as expected.”** This includes all four
wheels following all six directions, stopping after every input-expiry pause,
Space and quit, and remaining still through reset/startup and both unarmed W
checks. The application’s supervised stand scope passes.

Rover on stand, wheels clear, motor bank awake, USB connected. The actual
`python3 -m apps.teleop --target 192.168.4.1:7777` process received terminal
bytes through a pseudo-terminal. No separate UDP driver supplied commands.
Client source: `d5117db`; [identity](identity.json) records the full commit and
local normal-image digest. No flash: previously restored normal ROS image,
source `b12cc21`, SHA-256
`514ba816d3b6b8b3b2c40ee63e156fd4f07ca9be104cef8246d6e848602c73f3`.
Opening capture caused the single announced reset.

## Results

All timestamps below are firmware `at_ms`; they describe logged commands,
not measured physical stopping times. Wheel order is FL, FR, RR, RL.

| Stage | Serial/client evidence | Physical observation |
| --- | --- | --- |
| Startup/unarmed W | Claim at 4,624 ms, zero wheels; five-second idle and one-second W produced no ARM/nonzero state | Still |
| R at zero | Armed zero at 10,638 ms; matching client Armed reply; motion waited for the separate five-second announcement | Still |
| W forward | `83,83,83,83` at 32,992 ms; zero at 35,208 ms | Correct; stopped in pause |
| S backward | `-83,-83,-83,-83` at 35,988 ms; zero at 38,204 ms | Correct; stopped in pause |
| A strafe left | `-83,83,-83,83` at 39,006 ms; zero at 41,225 ms | Correct; stopped in pause |
| D strafe right | `83,-83,83,-83` at 42,027 ms; zero at 44,246 ms | Correct; stopped in pause |
| Q rotate left | `-32,32,32,-32` at 45,107 ms; zero at 47,324 ms | Correct; stopped in pause |
| E rotate right | `32,-32,-32,32` at 48,057 ms; zero at 50,284 ms | Correct; stopped in pause |
| Forward then Space | Forward at 51,086 ms; Stop reason=Stop at 52,079 ms, zero at 52,082 ms; matching STOP reply | Stopped |
| W without re-arm | One second of W after Space generated no nonzero command | Still |
| Explicit R then fresh W | Armed zero at 53,142 ms, fresh forward at 54,439 ms | Correct |
| Ctrl-D | Matching STOP acknowledgement; stop/release at 55,412 ms, Open zero at 55,416 ms; client exit 0 | Stopped |

[Raw serial](car.log), [client output](client.txt), [stage transcript](probe.txt)
and [result](result.json) retain the original evidence. The raw result says
physical observation pending because it was written before Todor replied;
this note records his subsequent confirmation without rewriting raw output.
The client correctly labels BYE/release unconfirmed; release is independently
visible in serial. One boot, one UDP claim/release, 19 state events and two Stop
transitions; no product fault, dropped event or panic. The bootloader’s existing
multiple-DROM-segments diagnostic remains as in the previous normal-image runs.

[Cleanup](cleanup.json) and [controller transcript](controller.txt) confirm
capture stopped, the previous Wi-Fi UUID restored, and the internet route
unchanged. No router or firewall rule was needed. The [terminal driver](driver.py.txt)
and [controller](controller.py.txt) preserve the exact executed scripts; their
hard-coded paths describe this run. The same terminal driver first passed the
entire sequence against the loopback product simulator.

## Scope

This accepts the interactive application on the stand with the normal firmware.
No loaded/floor result, measured speed, current, temperature margin, electrical
coast/STBY or physical stopping bound is claimed. The [M6 measurement deferrals](../../../openspec/changes/archive/2026-10-20-add-four-wheel-api/verification.md#task-52-disposition)
remain unchanged. Each future motor batch still requires its own announced go.
