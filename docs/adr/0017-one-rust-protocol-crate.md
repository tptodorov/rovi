---
status: proposed
---

# One Rust protocol crate is the wire contract for every participant

The car, the sim car, the laptop tools, the conformance suite and the phone each encoded the protocol separately (Rust, Python, JavaScript, byte slices). Todor decided (2026-10-28) that the Rust code defining the protocol is shared by all of them, so communication is strongly typed and verified the same way on sim and device.

* `crates/rovi-protocol` (`no_std`, no allocator, no I/O) defines every BLE, UDP and ROS message, identifier, code and the version, with validating symmetric codecs and direction-typed messages.
* Other languages use bindings of that crate (Python extension, WebAssembly), never their own codecs.
* Generated golden vectors are committed and checked in CI; changing bytes needs an explicit compatibility decision.
* One conformance suite runs through the shared types on the sim car and on the real car.

Details: [`add-rovi-protocol`](../../openspec/changes/add-rovi-protocol/design.md).

## Considered options

* **Schema language with code generation (protobuf, FlatBuffers).** Rejected: the BLE/UDP layouts and ROS CDR are fixed; a schema adds a generator to the firmware build without removing custom encodings.
* **Per-language codecs checked against shared vectors.** Rejected: four implementations of validation, drift caught late.

## Consequences

* Python and phone builds need a native extension and a WebAssembly module.
* Wire code leaves `rovi-core`; device logic stays there.
