# Legacy Python control stack

**Status:** current working product, but transitional — expected to be replaced by the ESP32-S3/Rust firmware once a milestone proves closed-loop driving end-to-end (see the root [README.md](../README.md#hardware-milestones)).

The main Rovi product is the Raspberry Pi application, meant to run with Python 3.11 (pinned in `.rtx.toml`). Its service entrypoint is [`roviservice.py`](roviservice.py):

```bash
cd legacy-python
pip install -r requirements.txt
python roviservice.py
```

The 4 motors are controlled via PWM pins connected to the motor drivers.
For each motor, one pin is used to control the direction and another pin to control the speed.

Implemented control methods:

* over Bluetooth PS4 controller directly connected to the car
* network via websockets from any internet host using a web page
* network via Zenoh from the local network using a keyboard

## Layout

* [`rovi/`](rovi/) — hardware control: motor driver (GPIO), PS4 controller input, Zenoh receiver.
* [`remote/`](remote/) — remote-control interfaces: websocket/web UI ([`README.md`](remote/README.md) has deployment notes), keyboard-over-Zenoh publisher.
* `event.py` — shared `ControlEvent` enum, kept at this top level (rather than inside `rovi/` or `remote/`) so both packages can import it without a circular dependency.
