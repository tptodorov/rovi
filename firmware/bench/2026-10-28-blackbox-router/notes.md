# 2026-10-28: the car against the ROS router on blackbox (bare board, no motion)

Todor connected the board over USB and said it could be used for testing; I ran only bare-board, disarmed tests (no ARM, no ROS stream, no motor power). Images built from `feat/network-mode-firmware` with `--features ros` (home settings from gopass `env/rovi/home`; the home SSID is redacted to `<home-ssid>`; the passphrase appears in no log). Router: `rovi-ros-router` on blackbox (`rmw_zenoh_cpp` 0.2.11 in Docker, `~/mycfg`), `tcp/192.168.1.175:7447` on its cable. Serial captured from a fresh boot per run with `espflash monitor`.

## Verified on blackbox (no car)

* **Restart across a crash:** `docker kill`: back in ~11 s (10 s restart delay + start). SIGKILL of the service: restarted, exactly one container (the orphan is removed by `preStart`), new router id.
* **Restart across a reboot (Todor's go):** after `systemctl reboot` the service came up by itself with 0 restarts, same image tag, listening, firewall rules present, and the **same DHCP addresses** (`.175` wired, `.165` Wi-Fi). Only `nscd` hit its start limit at boot (a name cache; lookups work).
* **LAN reachability:** the first firewall opened Wi-Fi only; the cable carries the LAN, so the car would have been refused (fix: both interfaces, generation 407).

## The car could not reach the router (found, root-caused, fixed)

First runs of the home image against blackbox: the car joined the Wi-Fi and got an address, then the **router step timed out** (0 of 12 boots; one earlier run with extra pings succeeded by chance). A ROS client on blade connected to the same router fine.

| Experiment | Result |
| --- | --- |
| Same image, router on blade over Wi-Fi (A/B) | works: router step 1408 ms, a series of 512-byte declaration batches ([ab-blade](ab-blade.log)) |
| Same image, router on blade's **wired** address, wired path forced | works, 1397 ms ([ab-wired](ab-wired.log)) |
| Capture on blackbox's cable during failing boots | ARP request from the car, ARP reply from blackbox, then often **no SYN**; once SYN + zenoh handshake + the first 512-byte token batch, then silence |
| Capture on **both** blackbox interfaces ([flux-boot](flux-boot.log)) | the car's one ARP request for `.175` was answered by **both** interfaces: cable `58:47:ca:77:8e:00` and Wi-Fi `dc:97:ba:41:0f:c3`. The car kept the later reply, and its SYN went to blackbox's **Wi-Fi** MAC, where strict reverse-path filtering drops it |

Root cause: **ARP flux** on a host with two interfaces on one subnet (Linux default `arp_ignore=0`). Fix in `~/mycfg` (blackbox, branch `fix/rovi-router-firewall`, generation 408): `arp_ignore=1`, `arp_announce=2` (persistent). Before the declarative change the same sysctls were set live: **4/5** boots succeeded, router step ~105 ms (previously never). After it, with the scan retry below: **10/10**, router step 103-107 ms (once 969 ms). The earlier theories ruled out on the way: laptop pings (0/5), Wi-Fi power-save (the driver default is already off), the NIC (gigabit, full duplex, no errors), the router version (identical to the one that works).

## Second finding: a single Wi-Fi scan missed the network (~1 in 30 boots)

[series-plain-3](series-plain-3.log): `Scan Failed` in 162 ms, so the car fell back without trying the router. The scan step now tries up to 3 times inside the core's 1.5 s budget. For a network that does not exist ([nossid-retry](nossid-retry.log)): scan fails after 457 ms, decision at 886 ms, reset-based switch, car AP configured about 1.8 s after the scan started (limit 2.5 s). The laptop-side timing of the AP appearing (12.9 s) is dominated by my own scan loop and is not a measurement of the car.

## Mode commands over BLE on the board ([ble-modes](ble-modes.log))

Fresh boot to `Home` at 4317 ms. BLE STOP-claim, then Control **4** (Car network): `mode_cmd Ble CarNetwork Ok`, `pending=car-network` readable over BLE, release, `Fallback reason=Requested`, reset (`Sw`). In fallback: BLE claim, Control **3** (Home): `mode_cmd Ble Home Ok`, `pending=home`, release, reset, `Home` again 2.2 s after boot. Also (same session): a BLE watcher read `mode=home ip=…` without claiming, UDP `INFO` from a non-owner answered, a BLE claim took the car and the station went off (UDP unanswered) ([final-home](final-home.log) is the end state: `Home`, router step 103 ms). From blade through `192.168.1.175` the graph shows `/rovi` and `/rovi/{arm,capabilities,echo,echo_reply,mode,network,release,status}`.

## Not done / open

* No motion, no ROS stream, no ROS-side mode command or release, no takeover with an armed owner: those are check 6.2 (needs Todor's announced go and the stand).
* **DHCP reservation is not possible:** the devolo is only a powerline-to-Wi-Fi bridge; the DHCP server is the gateway at `192.168.1.1` (Todor has not said what it is). Observed: lease time 24 h, the same addresses after a reboot. Options in the notes of the PR: static address on blackbox, or resolve the router by name ([#115](https://github.com/tptodorov/rovi/issues/115)).
* The `nixos-rebuild` switches on blackbox report a failed activation each time because `home-manager-todor` waits for an ssh passphrase prompt on its screen and `minidlna` aborts on restart. Both happened before this work.
