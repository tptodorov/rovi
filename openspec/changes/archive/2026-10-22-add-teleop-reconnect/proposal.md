# Proposal

## Why

A missed Echo deadline exits the keyboard client even when cleanup can still
reach the car. Todor requested automatic reconnection and a laptop beep on loss.

## What Changes

- Clear motion on loss, attempt bounded cleanup, and retry HELLO after a one-second
  delay until the car is available or the operator quits.
- Reconnect disarmed; require a matching fresh Echo and a new R before motion.
- Play a brief laptop tone once per outage; report unavailable audio and attempt
  the terminal bell when direct audio cannot play.
- Preserve gentle speeds, deadline checks, firmware and wire protocol.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `laptop-teleoperation`: recoverable connection failures and audible loss alert.

## Impact

Python session/runtime, tests and usage documentation only. Direct audio uses
the laptop's existing PipeWire player with a generated tone, without new Python
dependencies or system configuration. The underlying Wi-Fi fault (#23) remains
unresolved. Motor/radio acceptance requires a separately announced board test.

## Acceptance

Todor explicitly requested this behavior on 2026-10-20: “it should keep trying
to reconnect and establish connection” and “a beep from the laptop when
connection drops”. That authorizes this implementation. Agent review selects
disarmed recovery with fresh operator intent; no automatic ARM or motion replay.
