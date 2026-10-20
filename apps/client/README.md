# Shared client payloads

`wire.py` is the single Python implementation of TwistStamped CDR, BLE setpoints,
Status and Capabilities. From the repository root, import `apps.client.wire`.
The former `apps/simcar/scenarios/wire.py` path re-exports these codecs for existing
bench, simulator and ROS scripts. No installation or runtime dependency is needed.
Rust golden-vector/code checks remain in `apps/simcar/scenarios/test_wire.py`.
