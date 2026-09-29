# Rovi vision

Rovi is a modular robotics hardware + software platform for 4-wheel-drive vehicles, built one layer at a time:

1. **Hardware platform** — a 4-wheel-drive chassis and electronics that can independently drive 4 wheels.
2. **Low-level control API** — firmware that drives the 4 wheels independently, using as little CPU as possible, so most of the chip's budget stays available for whatever runs on top.
3. **Applications** — built on the low-level API. The first is a teleoperated 4-wheel mecanum car (PS4 controller / websocket / keyboard, plus a video stream). A future application could be autonomous driving; a future *platform* could be a boat or a flying device, reusing layers 1-2 where the physics allows.

Each layer should stay usable without the layers above it. The low-level API in particular should not bake in mecanum-specific or car-specific assumptions it doesn't need — that's what keeps it repurposable.

For current status and the roadmap, see [`PLAN.md`](PLAN.md). For how development work actually happens, see the root [`README.md`](../README.md).
