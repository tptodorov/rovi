# Shared client payloads

`wire.py` adapts the `rovi_protocol` module (the [Python binding](../../bindings/python) of the
`rovi-protocol` crate, ADR-0017) to the names the Python tools use: TwistStamped CDR, BLE
setpoints, Status and Capabilities. Layouts and validation live only in the crate. From the
repository root, import `apps.client.wire`; the former `apps/simcar/scenarios/wire.py` path
re-exports it. `rovi_protocol` comes from the project `nix-shell`, or from
`bindings/python/module.sh` for Python outside it (e.g. the ROS containers).
