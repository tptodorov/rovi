# 2026-10-19: BLE/AP radio exclusion on the board

Todor authorized the announced check with “go”. Installed ROS product firmware
from the [read-only batch](../2026-10-18-ros-readonly/notes.md); no flash.
Probe source `4c76766` (PR #77). The probe never sent ARM; every wheel command
in [car.log](car.log) is `0,0,0,0`. [probe.txt](probe.txt) is its output.

| Check | Evidence (car ms) | Result |
| --- | --- | --- |
| BLE owner turns the AP off | BLE claim 8,277; `wifi ap=off` 8,289. A 15 s `nmcli` join of `Rovi M8` failed while zero setpoints held the claim. | Passed |
| Release restores the AP | Disconnect released 26,385; `wifi ap=on` 26,406. Laptop joined; UDP HELLO got ACK (claim 30,382). | Passed |
| UDP owner stops BLE advertising | No `Rovi M8` advert in an 8 s scan while UDP held the claim with zero setpoints. | Passed |
| Release restores advertising | BYE released at 38,422; the next scan found Rovi. | Passed |

Note: after the probe stopped its BLE setpoints, BlueZ's two-second disconnect
delay let the 1 s lease expire first (`LeaseExpired` at 25,276, claimed and
never armed, zero wheels) before the disconnect release. Expected, harmless.

No errors, drops or panics. Prior Wi-Fi restored, Ethernet internet route
unchanged, capture stopped. UDP/ROS shared-AP exclusion is a separate check.
