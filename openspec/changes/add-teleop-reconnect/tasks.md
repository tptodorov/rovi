# Tasks

## 1. Contract and reproduction

- [x] 1.1 Record Todor's explicit reconnect/beep request and the disarmed recovery boundary in proposal/design/specs. Validate all planning artifacts before implementation.
- [x] 1.2 Reproduce reported exit with a PTY/fake-peer test dropping Echo; the new recovery assertion fails with `teleop: Echo reply timeout` before production edits.

## 2. Recovery and alert

- [x] 2.1 Implement typed recoverable failures, paced retries, fresh correlation and disarmed recovery. Verify dropped Echo/control replies, unavailable/BUSY startup, stale ACKs, socket errors, buffered R/motion, quit during outage and simulator restart; retain fatal scheduler/terminal checks. **Done:** full 39-test client suite passes; actual test names in verification.md.
- [x] 2.2 Add a bounded audible loss alert and honest fallback; test one alert per outage, a new alert after confirmed recovery, playback/fallback failures, and document runtime audio requirements and retry controls. **Done:** AudioTests and outage-latch tests pass; local PipeWire playback succeeds; usage documents requirements and fallback. Human audibility remains pending.

## 3. Host verification and delivery

- [x] 3.1 Run client tests against fake peers and the real product simulator, strict OpenSpec/reader validation and authored whitespace checks. Map actual evidence in verification.md and open a PR with CI results. **Done:** 39 client tests, strict spec validation, all six reader tests/freshness and whitespace pass. PR [#96](https://github.com/tptodorov/rovi/pull/96) contains the change; CI results are tracked on that PR. Physical acceptance remains task 4.1.

## 4. Board acceptance

- [ ] 4.1 Announce a separate gentle stand/reconnect/audio check and obtain Todor's go; verify wheel stillness through loss/reclaim, fresh R after recovery, audible alert, operator quit and network cleanup. Host passes do not complete this task.

## Workflow follow-up

- Archive/sync only after scoped physical acceptance; preserve #23 and existing measurement deferrals.
