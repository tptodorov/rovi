# 2026-10-18: product BLE Status/Echo delivery

Firmware source `7c2e2df` (PR #68), non-ROS product build. Todor authorized the
announced flash and non-driving probe with “go”. Setup requested: motor bank
disconnected. No setpoint was sent; ARM without a setpoint was deliberately refused.
This records the BLE subset of task 5.2; full product acceptance remains pending.

Flashed ELF SHA-256:
`3e99eb836ff180d49324d7837b19d4366dece161178c05fad911521e34bad2cd`.
Port: `/dev/serial/by-id/usb-1a86_USB_Single_Serial_5C85037678-if00`.
Flash verification completed, then the monitor reset the firmware for boot capture.

| Check | Result |
| --- | --- |
| Boot | `ev=boot`, no `ev=error` or panic |
| Immediate Status before subscription | Claimed / no reason / clear error `[1,0,0]` |
| Invalid control | Refused; Status read and notification `[1,0,4]` |
| Echo lengths 0, 4 and 32 bytes | Identical notifications; invalid error retained |
| STOP | Status read and notification `[3,1,0]` |
| ARM without setpoint | Refused; Status read and notification `[3,1,3]` |
| Empty Echo after refused ARM | Empty notification; not-ready error retained |
| Final STOP and disconnect | Clear error, release, Open; wheels `0,0,0,0` |

## Client failure and correction

The original probe timed out on its first empty Echo notification, reproduced on
three connections ([first output](probe.txt), [retry](probe-retry.txt),
[traced retry](probe-traced.txt)). The [ATT excerpt](empty-echo-att.txt) confirms
an empty Write Request, a Write Response and an empty Handle Value Notification
on Echo handle `0x0012`. The firmware delivered the reply over BLE.

The installed Bleak BlueZ backend defaults to `AcquireNotify`. Its
`_register_notify_fd_reader` treats an empty `os.read` result as EOF and closes the
notification stream. The probe now selects the supported
`bluez={"use_start_notify": True}` D-Bus path for Echo; Status retains its default
path so its read cannot masquerade as the required notification. No firmware
change or second flash was needed.

The original full [probe](../ble_status_echo.py) then passed on a fourth connection
([successful output](probe-fixed.txt)). [car.log](car.log) records all four
connections: none armed, final state Open with zero wheel commands. The serial
monitor and packet capture were stopped. The board remains on this firmware.
Raw packet capture stays outside the repository because it also contains unrelated
Bluetooth traffic; only the Rovi Echo exchange is included here.

Not tested: motor outputs/motion, reset or power-loss safety with motor power,
physical AP exclusion (serial AP transitions alone do not prove it), UDP or ROS
delivery, reversals or thermal behavior. Task 5.2 remains unchecked.
