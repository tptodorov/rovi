---
status: accepted
---

# ROS 2 on the ESP32-S3 via zenoh-nostd and rmw_zenoh, with XRCE as the fallback

The car must be usable as a ROS 2 node without an onboard Linux computer. Two laptop spikes compared the options:

* **[S1](../research/s1-micro-ros-nano-ros-spike.md), micro-ROS/XRCE via nano-ros.** Works, but it is C plus a framework with blocking I/O, and there is no upstream S3 support.
* **[S2](../research/s2-zenoh-nostd-spike.md), zenoh-nostd.** Pure Rust and async on embassy-net, interoperates with stock ROS 2 Jazzy through `rmw_zenoh` over UDP, and costs +101 KiB of flash.

We decided (2026-10-06, M8) to implement the optional `ros` feature on a pinned fork of zenoh-nostd, connecting as a client to `rmw_zenohd`. The fork carries optional `defmt`, `embassy-sync` 0.8 and a liveliness-token API, and each patch is offered upstream.

The known gap is ROS graph visibility, which needs `rmw_zenoh` liveliness tokens. It is M8's first risk item. If it isn't solved early in M8, revisit this decision and switch to the XRCE path from S1.

## Considered options

* **nano-ros (XRCE).** Rejected as primary: it is C plus a framework, and there is no S3 support.
* **Direct FFI to eProsima's Micro XRCE-DDS client.** Kept as the fallback.
* **A host-side bridge node.** Rejected: it needs a computer in the loop.
