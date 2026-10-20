# Design

## Context

The current client intentionally raises a fatal error for any missed reply.
A PTY/fake-peer reproduction drops Echo and produces the reported timeout/exit.
This establishes client behavior, not the cause of radio delay or packet loss.

## Decisions

- Keep the existing per-attempt Session and input Policy. A small outer retry
  loop handles typed link failures (HELLO/control/Echo timeouts, BUSY and socket
  errors). Invalid capabilities, terminal loss, input flood and scheduler gaps
  remain fatal. This avoids treating a broken terminal as a recoverable network.
- Clear Policy before cleanup/alert, send zero/STOP/BYE best effort, then wait
  one second with quit input serviced. New attempts use fresh Session/Policy
  instances and monotonically advanced control sequences so old ACKs cannot
  satisfy a new HELLO. Retry respects ownership; BUSY never authorizes motion.
- Require a fresh matched Echo before exposing recovered controls. Discard
  buffered keys at recovery boundaries, preserving quit; directions and R typed
  during the outage cannot resume motion. A post-recovery R and direction are
  necessary. Keep one loss alert latched until Echo confirms recovery.
- Generate a short PCM tone with the standard library and play through existing
  `pw-play`. Playback is bounded and happens only after motion is disabled.
  Missing/failed audio reports a warning and falls back to terminal BEL. Muted
  hardware cannot be detected; an audible check remains physical evidence.
- Retry the selected UDP endpoint only. NetworkManager keeps responsibility
  for AP association; the application does not switch networks or reset firmware.

## Risks / Trade-offs

- One dropped Echo can still trigger recovery: the 500 ms safety policy is
  preserved, rather than masking a slow link by widening deadlines.
- Lost cleanup can leave the old owner until its lease/reclaim window ends;
  retries wait through BUSY. Reclaiming does not bypass device ownership.
- Automatic reconnection improves usability but does not prove Wi-Fi stability,
  wheel-stop latency or physical safety. Host tests and board evidence stay separate.

## Migration

The same command starts the client; retries replace the previous terminal exit.
Reverting the client commit restores fail-and-exit behavior. No device change.
