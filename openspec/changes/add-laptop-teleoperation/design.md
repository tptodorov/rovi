# Laptop teleoperation design

## Context

See the [proposal](proposal.md) for motivation and accepted laptop scope. The existing [car API](../../specs/car-control-api/spec.md) and [M6 evidence](../archive/2026-10-20-add-four-wheel-api/verification.md) are the boundaries. `apps/simcar/scenarios/wire.py` already supplies golden-tested TwistStamped, Status and capability codecs; its scenario `Client` is a test helper, not a product transport.

## Goals / Non-Goals

**Goal:** one small Linux terminal client whose policy can be tested with injected time and socket/terminal boundaries. **Non-goals:** a client framework, configurable bindings/speeds, multi-axis mixing, runtime firmware changes, station networking or proof of physical velocity/stopping distance.

## Decisions

* Use Python’s standard library (`socket`, `select`, terminal facilities, monotonic time), invoked from the repository as `python3 -m apps.teleop --target HOST:PORT`. No new runtime dependency or global installation. A browser UI or gamepad adds another input/runtime boundary without being needed for this first interface.
* Keep policy separate from terminal/socket effects: a current intent, expiry, local arm gate and pending request deadlines. A single event loop handles input/replies before a 50 ms stream tick; no thread or command queue. Reject leases below 200 ms rather than promise a faster supported host cadence. A 200 ms scheduling gap fails closed; this is host policy, not a measured wheel-stop bound.
* Use R for arm, WASD translation, QE rotation, Space STOP, Esc/Ctrl-C quit. Terminal bytes provide key repeats, not trustworthy key-up/focus events: each direction is a 250 ms request that needs renewal. Drain buffered input to the latest motion intent; stop/quit overrides that batch, and arming clears it. Never replay accumulated input after a scheduling gap. Zero intent keeps the owner’s lease live while idle/disarmed; Space sends STOP while retaining the session and zero stream.
* Share, rather than copy, the existing payload codec in a small `apps/client` Python package. Keep a thin compatibility import at the existing scenario module path so bench/ROS/experimental users retain their imports and golden tests. Add frame validation in the client boundary; avoid converting every existing test helper into a production client in the same change.
* Correlate every control reply by endpoint, session and a nonzero request sequence; preserve unsolicited sequence-zero stops separately. Use wrapping u32 setpoint ordering independently. Only one ARM may be pending. If a stopped notification arrives while ARM is pending, cancel that request locally; a delayed Armed reply cannot revive local motion. A later R sends zero and a new request. Do not retry ARM automatically.
* Echo checks only reachability. UDP has no observational owner-status read, and HELLO can claim after a reset. Therefore perform HELLO only once and label Status as last reported with its own age. A reset may leave a responsive car rejecting the old session; this client never claims it silently. A new operator-started process is required to reclaim after terminal/link failure.
* All connect/control waits and cleanup are bounded. On exit try zero, STOP, BYE; catch individual send failures so later cleanup still runs, then restore terminal state in an outer finally block. STOP success needs the matching state/reason acknowledgement; BYE has no release acknowledgement and remains best effort. No promise that the wheel stopped merely because a datagram was sent.

## Risks / Trade-offs

* Terminal repeat may begin after the 250 ms window, producing an initial pause → document this pulse/renewal behavior; deterministic expiry matters more than pretending key-up exists.
* Wi-Fi can silently stall ([#23](https://github.com/tptodorov/rovi/issues/23)) → missed Echo deadlines end the client, the device lease remains independent, and no reconnect is automatic. This change does not diagnose or close #23.
* UDP can lose a stop or release → bounded best effort and explicit unconfirmed reporting; retain the device watchdog. No application-level emergency-stop guarantee.
* Existing source/net and motor bench evidence does not qualify a new terminal application → simulator/PTY evidence first, then a separately announced gentle stand batch; no loaded/floor test or electrical measurement is implied.

## Migration and rollback

Add the client without firmware or API changes. Preserve the current codec import through a compatibility module and rerun its existing tests and simulator scenarios. Keep the normal ROS firmware installed. Rollback removes the new application and restores the codec location; no board image or protocol migration is needed.

## Scope decision

Todor selected the laptop client on 2026-10-20 after reviewing the concrete option in #89. Implementation uses the proposed keys, 250 ms intent window and gentle defaults; no new ADR is needed for these reversible client choices.
