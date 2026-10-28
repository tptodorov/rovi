# Router by name: board verification, 2026-10-28

Todor requested “i want to resolve the router by name” and authorized continued USB testing. The directory name was inherited from the previous session; this continuation ran on October 28 (host clock). These are bare-board/no-command boot and reconnect checks; motor acceptance is recorded separately.

Firmware resolves `tcp/blackbox.local:7447` to IPv4 before calling zenoh-nostd. Every connection attempt resolves again. embassy-net's DNS socket fits the existing six socket slots; no multicast-group subscription is needed because smoltcp uses legacy unicast-response mDNS. Other host names use DHCP DNS; literal IPv4 bypasses DNS. A lookup is bounded at 3000 ms and the home supervisor still ends the overall attempt at its existing deadline. No stale literal fallback is stored. Gopass now supplies the named endpoint.

| Check | Retained evidence | Result |
| --- | --- | --- |
| Name at boot, first run | [name-1](name-1.log) | mDNS 1306 ms; Home |
| Intermittent association failure | [name2-9](name2-9.log) | Failed Join before lookup; firmware now retries up to three times inside the existing overall budget |
| After join retry | [name3-11](name3-11.log) | mDNS 8 ms; Home |
| Missing name | [bad name](b-badname.log) | Lookup fails at 3000 ms; fallback Router within the join budget |
| Wrong Wi-Fi passphrase | [wrong passphrase](e-wrongpass.log) | Fallback Join; retries do not extend the core deadline |
| Normal named endpoint | [final](final-name.log) | mDNS 5 ms; Home |
| Router service restart | [restart](router-restart.log) | Link down at 12582 ms, fresh mDNS at 14590 ms, link up at 14708 ms: 2126 ms recovery; Home retained |
| Uppercase `.LOCAL`, before fix | [before](uppercase-before.log) | Settings accepted it but smoltcp sent ordinary DNS; repeated lookup failures and Router fallback |
| Uppercase `.LOCAL`, after lowercase normalization | [after](uppercase-after.log) | mDNS 10 ms, Router step 106 ms; Home |

The prior session reported 14/15 successful name boots before the association retry and 12/12 after it; only representative logs were retained. The continuation directly reproduced and fixed the uppercase-name failure, then flashed the normal lowercase-name image for stand testing.

Host checks cover valid/invalid names, label length, numeric addresses/ports, event formatting and the log catalog. A failing regression test also reproduced acceptance of leading-zero IPv4 octets that Rust's socket parser rejects; validation now rejects them before a firmware build can reach that parser.

Blackbox is running its declarative router service with explicit Docker stop (`docker stop -t 5`, `TimeoutStopSec=20`; mycfg `9480c4c`). The service is active with one established car connection. Earlier crash and reboot verification remains in the [blackbox evidence](../2026-10-28-blackbox-router/notes.md).

All retained logs were checked against the actual `ROVI_HOME_PASSWORD` value, rather than the gopass entry's first line. No passphrase was found; home SSIDs are redacted. Bootloader diagnostics already present in the image are retained.

Limits: an actual DHCP address change, wired-to-Wi-Fi router failover, and ordinary unicast DNS were not exercised. No multicast-reception or router-discovery capability is claimed. The 15-minute home-mode coexistence soak remains task 6.3.
