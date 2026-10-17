# Product specification standard

Rovi follows **OpenSpec's `spec-driven` workflow** for product development, starting with M6. The CLI version is pinned in [`package.json`](../package.json), dependencies in `package-lock.json`, and the workflow in [`openspec/config.yaml`](../openspec/config.yaml). Use the repository's CLI, not an independently upgraded global installation. [ADR-0015](adr/0015-native-openspec-workflow.md) records this decision.

Experiments retain their [milestone process](../experiments/README.md). Bench results inform product requirements; they do not automatically accept a new product implementation.

## One home for each kind of information

| Location | Meaning |
| --- | --- |
| `openspec/specs/<capability>/spec.md` | Accepted, implemented product behavior; populated by archive/sync after acceptance. |
| `openspec/changes/<change-id>/proposal.md` | Why, what changes, New/Modified Capabilities, impact and proposal acceptance status. |
| `openspec/changes/<change-id>/specs/<capability>/spec.md` | Proposed requirement deltas for each listed capability. |
| `openspec/changes/<change-id>/design.md` | Implementation boundaries, decisions, migration, risks and deferrable questions. |
| `openspec/changes/<change-id>/tasks.md` | Ordered implementation and verification checklist. |
| `openspec/changes/archive/` | Completed changes retained by the archive command. |
| `GLOSSARY.md`, `docs/adr/` | Shared terminology and durable decisions; link rather than duplicate them. |

`openspec/` is the only specification root. Older ADRs mentioning `spec/` are historical. Installed skills must use the current paths and workflow here. Architecture decisions remain in ADRs; `design.md` links them and explains the change-specific implementation. For product changes, supply all four planning artifacts, even when the design is short. Pure documentation/tooling work such as this migration does not need an artificial product change proposal.

## Writing requirements

* Use `## ADDED Requirements`, `## MODIFIED Requirements`, `## REMOVED Requirements` or `## RENAMED Requirements` for deltas. Each requirement uses `### Requirement: ...`, normative SHALL/MUST wording, and at least one `#### Scenario: ...` with WHEN/THEN; add GIVEN where needed.
* New capabilities start with a meaningful `## Purpose` (at least 50 characters). Baseline specs have `## Purpose` and `## Requirements`; delta files are not baseline specs.
* Keep new requirement descriptions within the pinned validator's 500-character limit. Put examples and edge cases in scenarios. MODIFIED blocks retain the complete requirement and all scenarios; REMOVED blocks explain the reason and migration.
* Specify observable behavior: units, bounds, invalid input, state transitions, recovery and transport exceptions where relevant. Scenarios must agree with each other and identify an observable result. Every requirement must be traceable to a test or explicit physical acceptance check.
* An OPEN item states what is unknown, how it will be settled, and which work it blocks. Unknown behavior cannot silently become an implementation choice. Unmeasured hardware bounds may remain open while independent host work proceeds, but cannot count as passed acceptance.
* Each numbered task is a checkbox with a completion check. Separate host evidence from physical evidence. Put archive instructions under an untracked `Workflow follow-up` section, since archive happens after the tracked work is complete.

## Workflow and gates

Run commands from the repository root (shown with the required RTK prefix):

```sh
rtk npm ci
rtk npm run openspec -- new change <change-id>
rtk npm run openspec -- instructions proposal --change <change-id>
rtk npm run openspec -- status --change <change-id>
rtk npm run spec:check
```

Use `instructions specs`, `instructions design` and `instructions tasks` for the other artifact templates. Generated instruction files are changed through the tool, not by hand.

1. **Draft:** write proposal → specs and design → tasks. Draft PRs may have explicit open issues; strict validation checks format, not correctness or acceptance.
2. **Ready to implement:** planning artifacts exist, strict validation passes, and human/agent review finds no contradictory behavior, unreachable scenarios or unresolved decisions affecting the selected implementation tasks. Todor's explicit acceptance authorizes product implementation. A merge counts as acceptance only when the PR explicitly proposes that acceptance; merging documentation alone does not. Record the acceptance evidence in the proposal/task 1.1; do not ask again once authorized.
3. **Implement:** preserve the specification contract, test bug fixes with a failing reproduction first, and check each task only after its completion check passes. Resolve a changed requirement through an updated proposal before implementing conflicting behavior.
4. **Accept:** tests and scenarios have recorded evidence, required physical checks have passed, and open items blocking this change are resolved. Deferred work has an explicit scope boundary; the change does not claim its results. Each physical motor run still requires the separately announced test and Todor's go.
5. **Archive:** after implementation and acceptance, apply the deltas with the CLI, preserving the change history. Review the resulting baseline and archived document links, purposes and scenarios, then validate again and include the result in a PR.

```sh
rtk npm run openspec -- archive <change-id>
rtk npm run spec:check
rtk npm run openspec -- validate --archived --strict --no-interactive
```

Do not move delta files directly into `openspec/specs/`. Relative links may need updating when files move into the baseline or archive; the CLI does not repair them. Do not bypass validation to obtain an archive success.

## Enforcement and upgrades

The `OpenSpec validation` CI job installs the locked CLI and runs strict validation for active changes and baseline specs, plus archived task completion. A green job establishes syntax checks only; review and physical acceptance remain separate gates. `openspec status` checks artifact presence, not semantic completeness.

Upgrade the pinned CLI and lockfile in a PR. Run strict validation and rehearse archive in a disposable copy; inspect the resulting baseline specs. Update this standard for deliberate workflow changes. [Upstream schema and templates](https://github.com/Fission-AI/OpenSpec/tree/main/schemas/spec-driven) explain the convention; the installed, pinned schema determines the tools used here.
