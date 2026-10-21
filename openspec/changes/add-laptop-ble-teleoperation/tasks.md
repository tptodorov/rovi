# Tasks

## 1. Contract

- [x] 1.1 Record Todor's explicit BLE keyboard request and complete proposal, design and deltas. Strict validation and agent review must pass before implementation. **Done:** request quoted in proposal; strict validation passes. Review includes native BLE correlation and composed PR #96 behavior.

## 2. BLE client

- [x] 2.1 Add lazy BLE CLI selection, discovery/validation and native GATT loop sharing Policy/input/audio. Test malformed/unsupported peers, startup stillness, six directions, control/readback races, expiry, stale generations/Echo, disconnect/retry, buffered keys, quit and bounded failed cleanup with a fake GATT peer. **Done:** 20 BLE tests pass; verification.md maps actual tests and red/green deadline/scheduler regressions.
- [x] 2.2 Document Nix-shell commands, MAC/name discovery, keys, ownership/AP exclusion, limits, reconnect and honest cleanup. Check CLI errors never connect and verify UDP still runs without Bleak. **Done:** README/help updated; actual CLI/PTY and conflicting-mode tests pass. Full suite passes on Python without Bleak; BLE subset also passes in the declared Nix shell.

## 3. Host delivery

- [x] 3.1 Run full client suite with product simulator, test with installed Bleak in Nix shell, strict specs/reader/link/whitespace checks, and open a stacked PR with mapped evidence and CI status. **Done:** 59 client tests and the 20 BLE tests with Bleak 2.1.1 pass; strict specs, all six reader tests/freshness, link/test mapping and whitespace pass. PR [#97](https://github.com/tptodorov/rovi/pull/97) is stacked on #96; CI results are tracked there. Task 4.1 remains separate and pending.

## 4. Board acceptance

- [ ] 4.1 Prepare and announce a gentle stand BLE keyboard/reconnect/audio batch; await Todor's go. Record real MTU, radio exclusion, wheel observations, stops and cleanup separately from host tests.

## Workflow follow-up

- Archive after scoped physical acceptance, after PR #96's prerequisite recovery change; reconcile composed baseline deltas and links through the pinned CLI.
