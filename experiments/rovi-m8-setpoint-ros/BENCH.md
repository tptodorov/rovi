# M8 bench runbook

Step by step for the day the board is back. Each phase says what to run, what a pass looks like and where the evidence goes (`bench/<name>/`, committed; only `bench/sim-*` and camera frames are gitignored).

**Status: nothing here has run on a board.** Every command was checked on the laptop (`--help`, builds, offline `nmcli`, and phases 2, 6 and 7 rehearsed against a standalone sim car with the board's windows). That is development evidence only ([ADR-0002](../../docs/adr/0002-simulation-never-proves-hardware.md)): tick an acceptance box only after it is seen on the board.

Run everything from `experiments/rovi-m8-setpoint-ros/`. Motor hardware stays disconnected.

## 0. Preflight (laptop, online)

Do this while the laptop still has internet: during the bench its only Wi-Fi radio is on the car AP.

```sh
cd experiments/rovi-m8-setpoint-ros
nix-shell ../../shell.nix                                  # or direnv; gives espflash, bleak, task
espflash --version; python3 -c 'import bleak; print(bleak.__version__)'
ls ~/export-esp.sh ~/.rustup/toolchains/esp                # Xtensa toolchain (nix cargo cannot build it)
docker build -q -t rovi-ros-sim sim                        # ROS 2 Jazzy + rmw_zenoh + rclpy, cached for offline use
(cd sim/simcar && cargo build --features ros)              # sim car, for reproducing a board failure
task m8:test                                               # host tests still green
```

* **AP password.** The firmware takes it at build time from `ROVI_WIFI_PASSWORD` (8-63 ASCII). Reuse M3's private file `~/.config/rovi/m3-wifi-pass`. Never print it or paste it into a log.
* **Laptop on the car AP.** One profile, static `192.168.4.2/24`, no gateway (the car has no DHCP, and its AP takes one station). NetworkManager keeps the passphrase in its root-only keyfile:

  ```sh
  nmcli connection add type wifi con-name rovi-m8 ifname wlp1s0f0 ssid "Rovi M8" \
    wifi-sec.key-mgmt wpa-psk wifi-sec.psk "$(< ~/.config/rovi/m3-wifi-pass)" \
    ipv4.method manual ipv4.addresses 192.168.4.2/24 ipv4.never-default yes ipv6.method disabled \
    connection.autoconnect no
  ```

  `nmcli --wait 15 connection up rovi-m8` joins (it replaces the home network on `wlp1s0f0`: the **dedicated** radio). The **shared** setup (M3's confound) is in phase 3.
* **Firewall (ROS only).** The car connects *to* the laptop's router on UDP 7448, and the NixOS firewall on `blade` is active, so open it for the session: `sudo nixos-firewall-tool open udp 7448` (undo: `sudo nixos-firewall-tool reset`). The sim rehearsals ran over loopback and never hit this.
* **Serial port.** Use the **CH343 UART** connector (USB `1a86:*`, M2/M3 used it as `/dev/ttyACM0`), and only that cable: once the native USB port (`303a:1001`) sees traffic, `esp-println` moves the log there and the CH343 goes silent. `ls -l /dev/serial/by-id/` shows which is which; your user needs `dialout` (it has it on `blade`).
* **Bluetooth.** `bluetoothctl show | rg Powered` is `yes`. One adapter (`hci0`), so the second-central BLE scenario stays skipped.

Paste into every bench terminal:

```sh
export D=$(date +%F)
export PORT=$(find /dev/serial/by-id -name 'usb-1a86_*' 2>/dev/null | head -1)   # the CH343 UART port; empty means no board
export ELF=firmware/target/xtensa-esp32s3-none-elf/release/rovi-m8-firmware
export ELF_ROS=firmware/target/ros/xtensa-esp32s3-none-elf/release/rovi-m8-firmware
cap() {  # cap <name> [elf]: reset the board and capture its serial log into bench/<name>/car.log (Ctrl-C ends)
  mkdir -p "bench/$1" && espflash monitor --port "$PORT" --non-interactive --elf "${2:-$ELF}" | tee "bench/$1/car.log"
}
router() {  # rmw_zenohd with an extra UDP listener on 7448, the controller script mounted
  docker rm -f rovi-router >/dev/null 2>&1 || true
  docker run -d --rm --name rovi-router --net host -v "$PWD/sim/ros_controller.py:/ros_controller.py:ro" rovi-ros-sim bash -c '
    source /opt/ros/jazzy/setup.bash
    sed "s#\"tcp/\\[::\\]:7447\"#\"tcp/[::]:7447\", \"udp/0.0.0.0:7448\"#" \
      /opt/ros/jazzy/share/rmw_zenoh_cpp/config/DEFAULT_RMW_ZENOH_ROUTER_CONFIG.json5 > /tmp/router.json5
    ZENOH_ROUTER_CONFIG_URI=/tmp/router.json5 ros2 run rmw_zenoh_cpp rmw_zenohd'
}
ros() { docker exec rovi-router bash -c "source /opt/ros/jazzy/setup.bash; $*"; }
```

Each phase uses its own `cap` run: `espflash monitor` hard-resets the board, so every `car.log` starts at `ev=boot`. After a reset the laptop must rejoin the AP (`nmcli --wait 15 connection up rovi-m8`).

## 1. Build and flash

Build both variants (in a subshell, so the Xtensa `PATH` stays out of the other terminals):

```sh
(source ~/export-esp.sh; PATH=~/.rustup/toolchains/esp/bin:$PATH; cd firmware
 export ROVI_WIFI_PASSWORD="$(< ~/.config/rovi/m3-wifi-pass)"
 cargo build --release --target xtensa-esp32s3-none-elf -Zbuild-std=core,alloc
 CARGO_TARGET_DIR=target/ros cargo build --release --features ros --target xtensa-esp32s3-none-elf -Zbuild-std=core,alloc
 xtensa-esp32s3-elf-size ../$ELF ../$ELF_ROS) | tee /tmp/m8-size.txt
espflash board-info --port "$PORT"
espflash flash --non-interactive --port "$PORT" "$ELF"           # phases 2-5 and 7; $ELF_ROS for phase 6
cap board-$D-boot                                               # Ctrl-C after ~10 s
```

Look for, in `bench/board-$D-boot/car.log`:

* `M8 board-only AP=Rovi M8 udp=192.168.4.1:7777 WPA2 channel=1 lease_ms=1000`, then `ev=boot ver=0.1.0 at_ms=<n>`.
* No `panicked`, no `ev=error`, no `ev=dropped`; the LED is off (Open). `python3 sim/carlog.py bench/board-$D-boot/car.log` exits 0.
* With `$ELF_ROS` and no router yet: `ros: router udp/192.168.4.2:7448 not reachable, retrying` and `ev=ros link=retry`, repeating.
* The AP: `nmcli -f SSID,CHAN device wifi list --rescan yes | rg 'Rovi M8'` shows channel 1.

If espflash cannot connect: hold BOOT, tap RST, release BOOT, retry. Copy `/tmp/m8-size.txt` (text/data/bss) into the results; the latest laptop numbers are text 653,493 B without `ros` and 816,841 B with.

## 2. UDP scenarios

Terminal 1: `cap board-$D-udp`. Terminal 2:

```sh
nmcli --wait 15 connection up rovi-m8
ROVI_TARGET=192.168.4.1:7777 ROVI_CARLOG=$PWD/bench/board-$D-udp/car.log BENCH_NAME=board-$D-udp sim/bench.sh
```

Pass: `scenarios.txt` ends `Ran 8 tests ... OK`, `latency.txt` has `loss_pct=0.0`, `carlog.txt` has no `PROBLEM` line, and the script prints `(exit 0)`. Takes about a minute (the claim-window test waits 10 s).

Then the sequence check, which no scenario covers (acceptance: non-increasing sequences are never applied):

```sh
ROVI_TARGET=192.168.4.1:7777 PYTHONPATH=sim python3 - <<'EOF'
import time, udp_scenarios as u
from wire import cdr
c = u.Client(); assert c.hello()[0] == u.ACK, "car not free"
c.seq = 99; c.send(u.SETPOINT, cdr(0.5)); time.sleep(0.05)   # seq 100: wheels 250
assert c.status(u.ARM) == (2, 0, 0)
c.send(u.SETPOINT, cdr(-0.5), seq=50); time.sleep(0.05)       # older: dropped
c.send(u.SETPOINT, cdr(-0.5), seq=100); time.sleep(0.05)      # not newer: dropped
c.send(u.SETPOINT, cdr(0.3)); time.sleep(0.05)                # newer: wheels 150
print(c.status(u.STOP)); c.send(u.BYE)
EOF
rg 'ev=state' bench/board-$D-udp/car.log | tail -4
```

Pass: `Armed wheels_pm=250,...` then `Armed wheels_pm=150,...`, never a negative wheel. The mecanum scenario's values in `car.log` should be exact: vx 0.5 gives `250,250,250,250`, vy 0.5 gives `-250,250,-250,250`, wz 1 gives `-100,100,100,-100`.

**Wi-Fi loss or firmware bug?** Read `car.log` before blaming the firmware.

* **Wi-Fi:** the client timed out (`recv` got `None`, "the car never became free") but the car log is consistent with what arrived (the claim is missing, or the events are right but late); `latency.txt` shows loss; `journalctl -u NetworkManager --since -10min | rg wlp1s0f0` shows a disconnect. A re-run passes. Re-run one scenario with `(cd sim && ROVI_TARGET=192.168.4.1:7777 python3 -m unittest -v udp_scenarios.Scenarios.test_bye_reopens)` (any test name); `sim/udp_scenarios.py` itself ignores test names.
* **Firmware:** the car log contradicts the protocol: wrong stop reason, claimed twice (`carlog.txt PROBLEM`), a panic, `ev=error`, `ev=dropped`, wrong wheel signs, or the failure repeats with 0 % echo loss. Reproduce it on the sim car as a failing test first.

## 3. UDP latency, dedicated vs shared radio

The board's echo is open to any peer, so this runs at any time. 600 probes is 30 s at 20 Hz.

```sh
mkdir -p bench/board-$D-latency
ROVI_TARGET=192.168.4.1:7777 python3 sim/latency.py -n 600 | tee bench/board-$D-latency/udp-dedicated.txt
```

**Shared radio** (M3's setup: the home network and the car AP time-share one Intel radio; M3's p95 doubled with power save off). Untried with these commands; `iw` is not installed, so take it from nixpkgs:

```sh
nix shell nixpkgs#iw -c sh -c 'sudo "$(command -v iw)" dev wlp1s0f0 interface add wlrovi type managed'
nmcli connection clone rovi-m8 rovi-m8-shared && nmcli connection modify rovi-m8-shared connection.interface-name wlrovi
nmcli connection up "$HOME_WIFI" && nmcli --wait 15 connection up rovi-m8-shared   # HOME_WIFI: the home profile (nmcli connection show)
ROVI_TARGET=192.168.4.1:7777 python3 sim/latency.py -n 600 | tee bench/board-$D-latency/udp-shared.txt
nmcli connection down rovi-m8-shared; nix shell nixpkgs#iw -c sh -c 'sudo "$(command -v iw)" dev wlrovi del'
```

Record: radio mode, band and channel of the home network, distance to the board, power save (`nmcli -f 802-11-wireless.powersave connection show rovi-m8`). Acceptance uses the dedicated number; the shared one only sizes the confound.

## 4. BLE

Terminal 1: `cap board-$D-ble`. Terminal 3 (HCI trace for Q18/Q19): `sudo btmon -w bench/board-$D-ble/hci.btsnoop`. Terminal 2, with the car free (Open, LED off):

```sh
export ROVI_BLE_ADDR=$(python3 -c "import asyncio, bleak; d = asyncio.run(bleak.BleakScanner.find_device_by_name('Rovi M8', timeout=15)); print(d.address if d else '')")
echo "${ROVI_BLE_ADDR:?car not seen: is it advertising?}"
python3 sim/ble_client.py info | tee bench/board-$D-ble/info.txt             # mtu=..., capabilities, status
python3 sim/ble_client.py drive --vx 0.2 --seconds 10 | tee bench/board-$D-ble/drive.txt
ROVI_CARLOG=$PWD/bench/board-$D-ble/car.log BENCH_NAME=board-$D-ble sim/bench.sh    # BLE scenarios only (no ROVI_TARGET)
python3 sim/latency.py --ble "$ROVI_BLE_ADDR" -n 600 | tee bench/board-$D-ble/latency-ble.txt
```

Then stop btmon and read the trace:

```sh
btmon -r bench/board-$D-ble/hci.btsnoop | rg -i 'connection interval|exchange mtu|mtu:|slave latency|peripheral latency|supervision' | sort | uniq -c
```

Pass and what to record:

* **Q18 (MTU):** `info.txt` shows `mtu=` ≥ 75 and the trace shows the Exchange MTU response. The client's number comes from a private bleak call (`_backend._acquire_mtu`); the trace is the independent check. Also note the car's own `ev=ble link=up ... mtu=` (see troubleshooting: it may log before the exchange).
* **Q19 (interval):** the connection interval from the trace (initial and after any update). This is BlueZ's choice; the iPhone value stays open until an iPhone client exists.
* **20 Hz streaming:** `drive.txt` says `sent ~200 setpoints` for 10 s, statuses end `Stopped`, and the car log has no `LeaseExpired` during the drive. The car does not log each setpoint, so the delivered rate is inferred: no lease expiry plus 0 % loss on the 20 Hz echo.
* **Scenarios:** `ble-scenarios.txt` shows 10 pass and 1 skipped (`second_central`: it needs `ROVI_BLE_ADAPTER2`, a second adapter the bench doesn't have). Each BLE claim also turns the AP off (phase 5), so the laptop drops off Wi-Fi meanwhile.
* **Echo:** `latency-ble.txt` (median, p95, max). It owns the car for the whole run.

## 5. Wi-Fi AP off and on around a BLE session (Q20)

Terminal 1: `cap board-$D-q20`. Terminal 2, 10 cycles of BLE claim, release, AP rejoin, UDP check:

```sh
for i in $(seq 10); do
  python3 sim/ble_client.py drive --vx 0.2 --seconds 5 | tail -1
  t=$SECONDS; until nmcli --wait 15 connection up rovi-m8 >/dev/null 2>&1; do sleep 1; done
  echo "cycle $i: rejoined in $((SECONDS - t)) s"
  ROVI_TARGET=192.168.4.1:7777 python3 sim/latency.py -n 20
done 2>&1 | tee bench/board-$D-q20/cycles.txt
rg 'ev=(claimed|released|wifi|ble|error)|panicked' bench/board-$D-q20/car.log
python3 sim/carlog.py bench/board-$D-q20/car.log
```

Meanwhile, during a cycle, `nmcli -f SSID device wifi list --rescan yes | rg 'Rovi M8'` should find nothing.

Then the other direction (BLE off while UDP owns the car) with the car free and the laptop on the AP:

```sh
ROVI_TARGET=192.168.4.1:7777 PYTHONPATH=sim python3 - <<'EOF' | tee bench/board-$D-q20/ble-off.txt
import asyncio, threading, time, udp_scenarios as u, ble_client
from wire import cdr
c = u.Client(); assert c.hello()[0] == u.ACK, "car not free"
done = threading.Event()
def hold():  # setpoints keep the claim and the lease
    while not done.is_set():
        c.send(u.SETPOINT, cdr(0.0)); time.sleep(0.05)
threading.Thread(target=hold, daemon=True).start()
seen = asyncio.run(ble_client.find(timeout=8))
print("BLE while UDP owns:", "SEEN: fail" if seen else "not advertising: pass")
done.set(); time.sleep(0.1); c.send(u.BYE); time.sleep(0.5)
seen = asyncio.run(ble_client.find(timeout=8))
print("BLE after BYE:", f"advertising ({seen.address}): pass" if seen else "NOT SEEN: fail")
EOF
```

Pass, every cycle: `claimed source=Ble`, `wifi ap=off`, ..., `ble link=down`, `released`, `wifi ap=on`; the drive completes without a BLE error (the switch does not drop the BLE link); the AP is back and the UDP echo has 0 % loss; no `ev=error src=wifi_ap`, no panic. Record the rejoin times.

**If it is unreliable** (a fault, a dropped BLE link at the switch, an AP that does not come back, or a hang): take PLAN Q20's fallback, arbiter-only exclusion. The AP stays up during BLE sessions, exclusivity rests on the arbiter (`BUSY` for a second UDP `HELLO`), and BLE and the AP are separate sessions. That is a firmware change (drop the Wi-Fi gate in `firmware/src/fw.rs`) and a new ADR; record the failing log first.

## 6. ROS 2 (`--features ros`)

Flash `$ELF_ROS`, open the firewall (phase 0), check `ss -tlnp | rg ':7447'` shows nothing, then terminal 1: `cap board-$D-ros "$ELF_ROS"`. Terminal 2:

```sh
nmcli --wait 15 connection up rovi-m8
router; sleep 3; ss -ulnp | rg 7448                     # the UDP listener is up
# within a few s the car log shows "ros node /rovi up, zid ..." and ev=ros link=up
ros ros2 node list | tee bench/board-$D-ros/q2.txt                       # /rovi
ros ros2 topic info -v /cmd_vel | tee -a bench/board-$D-ros/q2.txt       # one subscription, node rovi, BEST_EFFORT, depth 1
ros ros2 topic info /rovi/status | tee -a bench/board-$D-ros/q2.txt      # Publisher count: 1
ros 'timeout 6 ros2 topic pub -r 20 /cmd_vel geometry_msgs/msg/TwistStamped "{twist: {linear: {x: 0.2}}}" >/dev/null & sleep 2; timeout 3 ros2 topic echo /rovi/status --once; wait' | tee -a bench/board-$D-ros/q2.txt
```

Q2 passes when `/rovi` is listed, `topic info` matches, and plain `ros2 topic pub` (no `-w 0`) claims the car: `ev=claimed source=Ros` and status `state=claimed`. Plain CLI cannot arm it: a second `ros2 topic pub /rovi/arm` is another node, which the car ignores by design (seen on the sim car).

The ROS scenario (one node with `/cmd_vel` and `/rovi/arm`, plus an intruder node), once the car log shows `ev=released` (lease plus reclaim window, about 6 s after the pub ends):

```sh
off=$(wc -l < bench/board-$D-ros/car.log)
ros python3 /ros_controller.py | tee bench/board-$D-ros/ros-status.txt
tail -n +$((off + 1)) bench/board-$D-ros/car.log > bench/board-$D-ros/ros-run.log
for p in 'ev=claimed source=Ros' 'state=Armed wheels_pm=500,500,500,500' 'ev=stop reason=Stop'; do
  rg -q "$p" bench/board-$D-ros/ros-run.log && echo "ok: $p" || echo "FAIL: $p"; done
rg -q 'wheels_pm=-' bench/board-$D-ros/ros-run.log && echo "FAIL: intruder drove" || echo "ok: intruder ignored"
rg -q 'state=armed' bench/board-$D-ros/ros-status.txt && rg -q 'state=stopped' bench/board-$D-ros/ros-status.txt && echo "ok: status"
```

**Reconnect** (router restart, three times):

```sh
for i in 1 2 3; do router >/dev/null; t=$SECONDS
  until ros ros2 node list 2>/dev/null | rg -qx /rovi; do sleep 1; [ $((SECONDS - t)) -gt 60 ] && { echo "FAIL $i"; break; }; done
  echo "restart $i: /rovi back after $((SECONDS - t)) s"; done | tee bench/board-$D-ros/reconnect.txt
```

**Stale session** (the car loses the link while the router keeps its session): drop the laptop off the AP for 5 s.

```sh
nmcli connection down rovi-m8; sleep 5; nmcli --wait 15 connection up rovi-m8
t=$SECONDS; until ros ros2 node list 2>/dev/null | rg -qx /rovi; do sleep 1; done; echo "back after $((SECONDS - t)) s" | tee bench/board-$D-ros/stale.txt
```

Pass: `/rovi` returns each time; the car log shows `ros session ended (<reason>)`, `ev=ros link=down`, retries, then `link=up`; no panic, `carlog.py` clean. A BLE claim on this build also drops the ROS session (the AP goes off); it must rejoin the same way after release. Then `docker rm -f rovi-router` and `sudo nixos-firewall-tool reset`. To show UDP is unchanged with `ros`, run phase 2 on this build too: `cap board-$D-ros-udp "$ELF_ROS"`, then `ROVI_BLE_ADDR= ROVI_TARGET=192.168.4.1:7777 ROVI_CARLOG=$PWD/bench/board-$D-ros-udp/car.log BENCH_NAME=board-$D-ros-udp sim/bench.sh` (the empty `ROVI_BLE_ADDR` skips the BLE leg).

## 7. Lease, claim and reclaim timing (car log)

Run right after phase 2, with its capture (`board-$D-udp`) still open, so one `car.log` holds the probes and the window scenarios. 10 probes take about 70 s:

```sh
ROVI_TARGET=192.168.4.1:7777 PYTHONPATH=sim python3 - <<'EOF'
# The last setpoint changes the wheels, so its ev=state marks its arrival (at most one 10 ms tick late).
# Then silence: the lease expires and the reclaim window runs out.
import time, udp_scenarios as u
from wire import cdr
for i in range(10):
    c = u.Client(); assert c.hello()[0] == u.ACK, "car not free"
    c.send(u.SETPOINT, cdr(0.5)); time.sleep(0.05)
    assert c.status(u.ARM) == (2, 0, 0), "arm refused"
    time.sleep(0.05); c.send(u.SETPOINT, cdr(0.3))
    time.sleep((u.LEASE + u.RECLAIM) / 1000 + 0.5); c.s.close()
    print(f"probe {i + 1} done", flush=True)
EOF
```

Then measure on the car's clock (`at_ms`):

```sh
PYTHONPATH=sim python3 - bench/board-$D-udp/car.log <<'EOF' | tee bench/board-$D-udp/windows.txt
import sys, carlog
for b, seg in enumerate(carlog.segments(carlog.read(sys.argv[1])), 1):
    claimed = lapsed = None
    armed, prev = False, [None, None]  # the last two non-Stopped state events
    for e in seg:
        n, t = e["ev"], e["at_ms"]
        if n == "claimed":
            claimed, lapsed, armed = t, None, False
        elif n == "state" and e["state"] != "Stopped":
            prev = [prev[1], e]
            if e["state"] == "Armed":
                armed, lapsed = True, None
        elif n == "stop" and e["reason"] == "LeaseExpired":
            lapsed = t
            p, q = prev
            if q and p and q["state"] == p["state"] == "Armed":  # a wheel change while armed: a setpoint's arrival
                print(f"boot {b} lease    {t - q['at_ms']:6} ms  last setpoint -> LeaseExpired")
        elif n == "stop":
            lapsed = None
        elif n == "released" and claimed is not None:
            if lapsed is not None:
                print(f"boot {b} reclaim  {t - lapsed:6} ms  LeaseExpired -> released")
            elif not armed and t - claimed >= 1000:  # BYE and free-car probes release at once
                print(f"boot {b} claim    {t - claimed:6} ms  claimed -> released, never armed")
            claimed = None
EOF
```

Pass: every `lease` row ≤ 1040 ms (the true delay is the row plus up to one 10 ms tick, so this meets lease + 50 ms), every `reclaim` row about 5000 ms and every `claim` row about 10000 ms (each plus up to a tick). Lease rows come only from the probes: in the scenarios the last wheel change is the ARM, not the last setpoint. On the sim car this gave lease 995-1003, reclaim 5002-5009 and claim 10000-10001 ms.

## 8. Record the results

Check the passphrase leaked nowhere, then commit the `bench/board-*` folders:

```sh
rg -lF "$(< ~/.config/rovi/m3-wifi-pass)" bench/ && echo "LEAKED: scrub before committing" || echo "clean"
```

Paste into MILESTONE.md under Results:

```markdown
#### Bench: board-only, <date> — <pass/fail>

* **Setup:** commit `<sha>`; board ESP32-S3 rev <x> (`espflash board-info`), CH343 port; espflash <v>, BlueZ <v>, bleak <v>, NetworkManager <v>; laptop radio dedicated|shared (home network band/channel), distance <m>; motors disconnected.
* **Footprint:** text/data/bss <n>/<n>/<n> B, with `ros` <n>/<n>/<n> B.

| Phase | Result | Numbers | Evidence |
| --- | --- | --- | --- |
| 1 Boot | | `ev=boot` at <n> ms | `bench/board-<date>-boot/` |
| 2 UDP scenarios + sequence | | 8/8, echo loss <n> % | `bench/board-<date>-udp/` |
| 3 UDP latency | | dedicated <med>/<p95>/<max> ms; shared <...> | `bench/board-<date>-latency/` |
| 4 BLE | | MTU <n> (Q18), interval <n> ms (Q19), 10/11 + 1 skipped, echo <med>/<p95>/<max> ms | `bench/board-<date>-ble/` |
| 5 AP switch (Q20) | | <n>/10 cycles clean, rejoin <n>-<n> s | `bench/board-<date>-q20/` |
| 6 ROS | | Q2 yes/no, scenario, 3/3 restarts, stale rejoin <n> s | `bench/board-<date>-ros/` |
| 7 Windows | | lease <min>-<max> ms, claim <n>, reclaim <n> ms | `windows.txt` |

* **Not run or not observed:** <list>
```

Each acceptance box and what proves it:

| Acceptance item | Proof | Stays unticked if |
| --- | --- | --- |
| A BLE, UDP or ROS claim makes it the owner; the other radio shuts down and reopens | phase 2 `test_claim_busy_*`, phase 4 claim scenario, phase 6 `ev=claimed source=Ros`; phase 5 cycles (AP off/on) and `ble-off.txt` (advertising off/on) | Q20 fails (the item is then re-specified by the fallback ADR) |
| A second claimant is refused; non-owner ROS gid ignored | phase 2 `BUSY`; BLE: no advertising while owned (`ble-off.txt`, BLE disconnect scenario); phase 6 intruder check | — (a second BLE central is not tested: one adapter; note it) |
| New owner disarmed, only ARM enables, every stop disarms | phase 2 and 4 scenarios (arm needs a fresh setpoint, stop, lease expiry), phase 6 status | — |
| Latest wins, non-increasing sequences never applied | phase 2 sequence check | — |
| Lease expiry within lease + 50 ms; claim and reclaim windows | phase 7 `windows.txt`, phase 2/4 window scenarios | any lease row > 1040 ms |
| Wheel commands match `mecanum_drive_controller` | phase 2/4 mecanum scenario plus the exact `wheels_pm` values in phase 2 | — |
| `ros2 node list`/`topic info` show `/rovi`; plain `ros2 topic pub` and stamped teleop drive it | phase 6 `q2.txt`, ROS scenario | **stays open:** CLI pub claims but cannot arm, and the image has no `teleop_twist_keyboard` (see below) |
| BLE setpoint works at the negotiated MTU at 20 Hz | phase 4 `info.txt`, trace, `drive.txt` | MTU < 75 (Q18 fallback: compact 16-byte setpoint) |
| Latency for BLE, UDP and ROS, and the footprint, recorded | phases 1, 3, 4 | **stays open:** ROS has no echo yet ([#16](https://github.com/tptodorov/rovi/issues/16)) |

Tick only what was seen on the board in this run. Untried even after a clean run: the iPhone (Q18/Q19 on iOS), a second BLE central, ROS latency, stamped teleop, Wi-Fi range and other OSes.

## Troubleshooting

* **Port 7447 or 7448 already taken.** Another `rovi-router` (the sim scripts in any worktree use the same container name) or a host `rmw_zenohd`: `docker ps`, `ss -tlnup | rg ':744[78]'`, then `docker rm -f rovi-router`. Don't run the sim ROS scripts during the bench.
* **The car never joins the router.** Car log repeats `router udp/192.168.4.2:7448 not reachable`: check the firewall (phase 0), the laptop address (`ip -4 addr show wlp1s0f0`), the UDP listener (`ss -ulnp | rg 7448`) and that the laptop is on the AP at all.
* **bleak finds nothing.** The car advertises only while free: not while UDP or ROS owns it, and not during a claim or reclaim window (wait up to 16 s, stop other clients). Check `bluetoothctl show` (powered) and that a BLE client is not still connected. After one "car not found", the remaining BLE scenarios fail at once.
* **ATT MTU readout.** `ble_client.py` uses bleak's private `_backend._acquire_mtu` (BlueZ) and falls back to the write size + 3; a bleak upgrade can break it. The btmon trace is the authority.
* **Car logs `mtu=23` but the client reports ≥ 75.** The firmware logs `ev=ble link=up` right at accept, likely before the client's MTU exchange; the BLE claim scenario's log check then fails. Not a Q18 failure: confirm with the trace and open an issue.
* **CH343 log goes silent.** A cable on the native USB port took the log; unplug it and reset.
* **"the car never became free".** A previous owner still holds it (BLE connected, ROS publishing, or a window running). Stop the other client and wait for `ev=released`.
* **Lost the AP mid-run.** The board reset (`cap` resets it) or the AP was off for a BLE session: `nmcli --wait 15 connection up rovi-m8`.
* **A short state is missing from the log.** `ev=state` is sampled by the 10 ms tick, so a state that lasts less than a tick may not appear (seen on the sim car); the client's status is authoritative for it.
* **No internet during the bench.** Expected with the dedicated radio; build the Docker image and both firmware variants in phase 0.
