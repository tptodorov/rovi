# Tasks

Laptop tasks are development evidence only ([ADR-0002](../../../docs/adr/0002-simulation-never-proves-hardware.md)). Bench tasks wait for the M4/M5 setup.

## 1. Before implementation (laptop)

- [ ] 1.1 Review and accept this proposal.
- [x] 1.2 Decide whether ROS 2 controllers get the capabilities: yes, on a latched `/rovi/capabilities` topic (a product requirement, decided 2026-10-11).
- [ ] 1.3 Create the Cargo workspace (PLAN defers it until now) and move M8's host-tested core into a product crate, keeping its tests green.

## 2. Motor control (laptop, test-first)

- [ ] 2.1 Motor control module with a host test double: four wheel commands, enable, standby, no dependency on the device layer.
- [ ] 2.2 Tests: per-wheel independence, clamping, non-finite command → standby and fault, standby until enabled.
- [ ] 2.3 Connect M8's `WheelOutput` to motor control; the sim car logs the commands it would drive.

## 3. Car control API conformance (laptop)

- [ ] 3.1 Map each `car-control-api` scenario to a host test or a sim car scenario (`sim/udp_scenarios.py`, `sim/ros-scenarios.sh`); add the missing ones.
- [ ] 3.2 Publish `/rovi/capabilities` and settle O9: test a late `ros2 topic echo` against Jazzy on the sim car; fall back to periodic republish if transient local does not work.
- [ ] 3.3 Implement the `/rovi/status` requirement: add the stop reason and the last command error, and publish on change as well as every second (M8 publishes only state and session, once a second).

## 4. Settle OPEN requirements (bench)

- [ ] 4.1 M4 settles O1, O4, O5 (brake), O7. Write the measured values into the spec.
- [ ] 4.2 M5 settles O2, O3, O5 (stopping distance), O6, O8. Write the measured values into the spec.
- [x] 4.3 Close PLAN Q11 from O3: open-loop duty control selected (2026-10-17); no encoders, added sensors out of scope, existing approximate motion references accepted. No closed-loop ADR needed.

## 5. Implement and accept (bench)

- [ ] 5.1 TB6612FNG motor control on four channels with the M4 limits.
- [ ] 5.2 Run every scenario on the car, on all four physical channels, with BLE, UDP and ROS 2 controllers.
- [ ] 5.3 Record results; move the specs from this change into `spec/specs/`.
