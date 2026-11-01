# Rename tasks

## 1. Approval

- [x] 1.1 Todor explicitly accepts the rename for implementation; record the wording. Done: the request of 2026-11-01 quoted in the proposal.

## 2. Implementation

- [x] 2.1 Rename the firmware's AP SSID, GAP name, scan response and boot log; switch the clients, observation harness, bench scripts and current docs. Check: tests pinning the name failed first and pass now, firmware builds with and without `ros`, sim scenarios and host checks pass. CI results are recorded in the PR.
- [x] 2.2 Verify the radio names without arming after flashing and updating the laptop profile SSID: BLE advertisement and GAP name are `Rovi`, Wi-Fi scan shows SSID `Rovi`, device observation conformance and laptop discovery pass, and the published phone picker requests `Rovi`. Done 2026-11-01: 5/5 observation rules pass; Home restored, zero arm events and all wheel samples zero. Phone build `8e84bdaac9c2` is live with its picker and wasm verified. The 46 built-page browser tests cover selection; a fresh physical Bluefy check and AP association were not performed. [Evidence](../../../../firmware/bench/2026-11-01-rovi-name/notes.md).

## Host evidence (2026-11-01)

- Name regression tests for the phone picker, laptop target and product bench discovery failed against the old name before implementation.
- Workspace Rust tests and clippy; firmware clippy and release builds, plain and `ros`, with a throwaway AP password; Rust formatting checks.
- UDP and network-mode sim scenarios (14 each), observation conformance (5), bench unit tests (48), teleop tests (81), phone Node tests (69), built-page Chromium tests (46).
- Strict active/baseline and archived OpenSpec validation, plus generated-reader checks. CI results will be available in the PR.
- On host `blade`, read-only `nmcli` reports SSID `Rovi M8` for profile `rovi-m8`; no matching declaration was found in `~/mycfg`. No configuration was changed.
- The implementation agent performed host checks only; subsequent device and publication evidence is recorded in task 2.2.

## Workflow follow-up

Archive after task 2.2; its scoped device and publication checks passed. Preserve the historical experiment names and recorded evidence.
