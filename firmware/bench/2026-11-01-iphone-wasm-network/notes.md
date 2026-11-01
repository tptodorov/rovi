# Bluefy check of the WebAssembly page, 2026-11-01 (add-rovi-protocol 4.2)

Motor power off, no ARM. Page build `712ef024b110` from GitHub Pages (wasm served as `application/wasm`); car firmware is the protocol stack's top build. Phone and Bluefy versions were not recorded.

## Result: pass

Todor's observations:

* The page connected.
* **Use car network** and **Use Home Wi-Fi** each disconnected Bluetooth. He first read that as lost connection; the page requires a manual reconnect by design.
* After tapping Connect again, the panel showed the expected Home Wi-Fi with nothing pending ("correct").

Serial ([car.log](car.log)):

* `mode_cmd source=Ble cmd=CarNetwork result=Ok`, release, restart into fallback with reason Requested.
* The phone reconnected and claimed; `mode_cmd source=Ble cmd=Home result=Ok`, release, restart, Home with the ROS router 2.2 s after boot.
* The phone reconnected again and claimed with its STOP. Zero arm events; wheels 0 throughout.

The capture was snapshotted while the phone was still connected, so its final disconnect is not in this log. The later motion recheck is recorded in the [stand evidence](../2026-11-01-protocol-stand/notes.md) (task 4.2). Home SSID redacted.
