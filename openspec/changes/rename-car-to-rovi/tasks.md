# Rename tasks

## 1. Approval

- [x] 1.1 Todor explicitly accepts the rename for implementation; record the wording. Done: the request of 2026-11-01 quoted in the proposal.

## 2. Implementation

- [x] 2.1 Rename the firmware's AP SSID, GAP name, scan response and boot log; switch the clients, observation harness, bench scripts and current docs. Check: tests pinning the name failed first and pass now, firmware builds with and without `ros`, sim scenarios and host checks pass. CI results are recorded in the PR.
- [ ] 2.2 Physical check without arming, after flashing and setting the laptop profile's SSID: the car appears as `Rovi` on BLE and Wi-Fi; `observe.py` with `ROVI_OBSERVE=device` passes; the laptop client and the published phone page find it. Record evidence under `firmware/bench/`.

## Host evidence (2026-11-01)

- Name regression tests for the phone picker, laptop target and product bench discovery failed against the old name before implementation.
- Workspace Rust tests and clippy; firmware clippy and release builds, plain and `ros`, with a throwaway AP password; Rust formatting checks.
- UDP and network-mode sim scenarios (14 each), observation conformance (5), bench unit tests (48), teleop tests (81), phone Node tests (69), built-page Chromium tests (46).
- Strict active/baseline and archived OpenSpec validation, plus generated-reader checks. CI results will be available in the PR.
- On host `blade`, read-only `nmcli` reports SSID `Rovi M8` for profile `rovi-m8`; no matching declaration was found in `~/mycfg`. No configuration was changed.
- No serial access, flash, physical BLE/Wi-Fi check or page publication was performed. Task 2.2 remains open.

## Workflow follow-up

After 2.2, archive this change. That moves the wording into the baseline specs, which keep the old name until then.
