# Rovi vision

## What Rovi is

Rovi is a modular robotics hardware + software platform for 4-wheel-drive vehicles, built one layer at a time:

1. **Hardware platform** — a 4-wheel-drive chassis and electronics (currently: ESP32-S3-N16R8 + TB6612FNG motor drivers) that can independently drive 4 wheels.
2. **Low-level control API** — firmware that drives the 4 wheels independently, using as little CPU as possible, so most of the chip's budget stays available for whatever runs on top.
3. **Applications** — built on the low-level API. The first is a teleoperated 4-wheel mecanum car (PS4 controller / websocket / keyboard, plus a video stream). A future application could be autonomous driving; a future *platform* could be a boat or a flying device, reusing layers 1-2 where the physics allows.

Each layer should stay usable without the layers above it. The low-level API in particular should not bake in mecanum-specific or car-specific assumptions it doesn't need — that's what keeps it repurposable.

## Current state (2026-09-29)

- **[`legacy-python/`](../legacy-python/)** is the current working product: a Raspberry Pi Zero 2 W application driving 4 motors directly over GPIO, controlled via PS4/websocket/Zenoh, with a video stream. It predates this layered vision and doesn't implement it — it's a flat, car-specific application. It stays the working product until the ESP32-S3 platform reaches parity.
- **[`experiments/`](../experiments/)** is where the ESP32-S3/Rust hardware platform track is being bootstrapped, one milestone at a time. **M1** (in progress) proves the smallest possible slice: one ESP32-S3 driving one motor through one TB6612FNG channel, in Rust `no_std`. See [`experiments/rovi-m1-wokwi/MILESTONE.md`](../experiments/rovi-m1-wokwi/MILESTONE.md).
- No code yet implements the low-level 4-wheel API or any layered application — that starts once M1 proves the basic chain works on real hardware.

## Hardware-validation policy

**Simulation never counts as proof for hardware a simulator doesn't model electrically.** Wokwi simulates ESP32-S3 GPIO/PWM timing but not the TB6612FNG's electrical behavior, so a passing Wokwi run only proves the firmware doesn't panic and toggles the right pins at the right times — it does *not* prove the motor driver stack works. A milestone touching hardware isn't accepted until it's been physically bench-verified. This applies to every future hardware component (drivers, sensors, etc.), not just this one.

## How work gets done here

Two different processes, by intent:

- **Exploratory hardware bring-up** (`experiments/`) stays lightweight: each milestone is an independent Cargo package with colocated `README.md`/`MILESTONE.md`/`WIRING.md` (see [`experiments/README.md`](../experiments/README.md)). Results get tracked, not just planned — after bench testing, the milestone's `MILESTONE.md` gets a `## Results` section (what was observed, pass/fail, date, links to logs/photos), and `experiments/README.md`'s index gets a status marker. No shared code between milestones yet — that starts with the low-level API below.
- **Product development** — the low-level 4-wheel API and anything built on top of it — follows spec-driven development (the OpenSpec convention: a change proposal + spec deltas + a task list under `spec/changes/{change-id}/`, via the `openspec-implementation` skill). This does not apply retroactively to `legacy-python/` or to exploratory experiments; it starts with the first real proposal, which will be the low-level API itself, once M1 proves the chain works on hardware.

## Explicit non-goals (for now)

These were considered and deliberately deferred — revisit only if the premise changes:

- **Autonomy** is not a committed near-term goal. Nothing here should be architected around it yet, but the low-level API shouldn't preclude it either.
- **Splitting the platform into its own repo** is deferred until a second real hardware application (boat, drone, etc.) actually needs to consume it independently of this car.
- **A Cargo workspace** for shared firmware code is deferred until the low-level API work actually starts — `experiments/` stays independent packages until then.

## Roadmap

- GitHub milestone tracker: https://github.com/tptodorov/rovi/milestones
- **M1** (in progress): prove one ESP32-S3 + one TB6612FNG channel can drive one motor, on real hardware.
- M2+ : not yet defined. The next candidate is the low-level 4-wheel API, once M1 lands.
