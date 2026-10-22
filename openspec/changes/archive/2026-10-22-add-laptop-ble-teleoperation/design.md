# Design

## Context

The UDP loop uses socket readiness and request-correlated frames. BLE instead
claims at connect and exposes async GATT writes, readable Status and notifications.
The existing bench BLE client establishes UUIDs, BlueZ MTU acquisition and codecs.

## Decisions

- Add a native async BLE loop beside UDP, sharing Policy, terminal input, codecs
  and audio. Avoid a local fake-UDP bridge or broad transport refactor: GATT has
  no UDP session/control sequence to correlate. UDP's tested loop stays intact.
- Use the installed Bleak dependency lazily. Bound discovery to five seconds
  (four-second scan plus setup/teardown margin), reject
  ambiguous name discovery, and connect with a five-second bound. A MAC pins
  every retry; name discovery must find exactly one matching advertisement and
  then pin its address. Recovery cannot roam to another advertising car.
- Validate service/characteristics/properties, capabilities and MTU/write capacity
  before motion. Require a matching Echo before enabling keys. Use the established
  BlueZ MTU acquisition hook, with advertised write capacity as fallback; fail
  closed if 72-byte writes cannot be established.
- Keep only latest intent and one pending control action, never a motion queue.
  Serialize GATT operations with short deadlines. ARM writes zero first and
  requires a successful ATT write plus fresh Armed Status read. Notifications
  never authorize ARM. Space/safety stops invalidate pending ARM confirmation.
- Tie callbacks and Echo nonces to a connection generation. Ignore closed-peer
  callbacks. Drop buffered keys at readiness/reconnect, preserving quit. A lost
  connection clears motion before cleanup, one loss alert and a one-second retry.
- Best-effort zero, STOP/readback and disconnect share a 500 ms cleanup budget;
  failed writes cannot skip disconnect. ATT success/readback is not wheel feedback.
  Bleak cancellation and terminal restoration also cover discovery/setup/quit.

## Risks and limits

BLE ownership turns off the car AP; no simultaneous UDP/BLE control or seamless
transport switching. Another owner prevents advertising, so discovery keeps
retrying without stealing ownership. Direct sound can be muted or unavailable.
Host fake-GATT tests cannot establish radio, MTU negotiation or physical stopping
acceptance; those remain a separately authorized stand check.

## API references

[Bleak client](https://bleak.readthedocs.io/en/latest/api/client.html) documents
write-with-response versus queued writes, characteristic capacity and BlueZ's
MTU reporting limitation. [Scanner](https://bleak.readthedocs.io/en/latest/api/scanner.html)
documents discovery with advertisement data. Local installed API is checked
through the project's Nix shell; no dependency upgrade is required.
