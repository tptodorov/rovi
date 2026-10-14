---
status: accepted
---

# The ROS 2 owner is a node, learned from the graph's liveliness tokens

[ADR-0004](0004-single-owner-latest-setpoint.md) gives the car one owner. Over ROS 2 a sample carries only its publisher's gid (a hash of the publisher's liveliness key), and a controller's `/cmd_vel` and `/rovi/arm` are two publishers with two gids. An owner keyed by gid therefore could not arm its own car; the ROS sim scenario found this on 2026-10-08. We decided (2026-10-08, M8):

* **The owner is the ROS node** (`<zid>/<nid>`) of the publisher, not the publisher.
* The car subscribes to the graph's publisher liveliness tokens (`@ros2_lv/<domain>/*/*/*/MP/**`) and maps each publisher gid to its node.
* Samples from publishers it has not seen announced are ignored, and they cannot claim. When one arrives, the car asks the router for the tokens again (at most every 2 s), because a token lost over UDP is never resent.

## Considered options

* **Owner = publisher gid.** Rejected: a node could not arm with a second publisher, and every topic would need its own claim.
* **Owner = the zenoh session (zid).** Rejected: one process can host several nodes, and the router's own session relays everything, so zid alone does not separate controllers.
* **An explicit claim message with an id in the payload.** Rejected: stock ROS tools could not send it.

## Consequences

* The car keeps a fixed-size publisher table (oldest entry evicted) and depends on liveliness tokens: a controller whose tokens never arrive cannot drive until the refresh brings them in.
* Two CLI commands (`ros2 topic pub` twice) are two nodes, so one cannot arm the other's stream. [ADR-0011](0011-ros-auto-arm.md) removes the need for that.
* Any node in the graph can claim a free car; the graph is trusted ([ADR-0008](0008-local-link-security-baseline.md)).
