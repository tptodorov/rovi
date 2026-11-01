# Motion rules on the stand, 2026-11-01 (add-rovi-protocol 3.4, 4.2)

Motor power on, wheels clear, Todor watching; he said "go for the next test". Firmware: `main` at `712ef02`, ROS + home settings, router `tcp/blackbox.local:7447`; ELF sha256 `9ab5c036f04ce566`.

## Laptop batch: pass ([stand.py](stand.py), [stand-ros.py](stand-ros.py), [car.log](car.log), [ros.log](ros.log))

* ROS stream with one-second forward pulses (`wheels_pm=83,83,83,83`).
* UDP takes over from ROS (`preempted`, `stop OwnerLost`, `claimed Udp`); unarmed setpoints stay at zero; explicit ARM pulse, STOP, BYE.
* BLE takes over from moving ROS; explicit ARM; deliberate Home hand-back lets ROS reclaim and auto-arm.
* BLE disconnect without Home: lease stop (BlueZ took longer than the lease to disconnect); ROS stays at zero until `/rovi/arm` true.
* Final ROS STOP and release. `/rovi/network` shows `reason=` on the real car ([ros.log](ros.log)).

## Phone on the WebAssembly page, build `712ef024b110`: pass ([phone-car.log](phone-car.log))

Connect, ARM AT ZERO, all six direction buttons, STOP while holding, Disconnect. Serial shows all six wheel patterns (`83,83,83,83`, `-83,-83,-83,-83`, `83,-83,83,-83`, `-83,83,-83,83`, `32,-32,-32,32`, `-32,32,32,-32`), stop, release, no resets.

## Operator observation

Todor, asked about the wheels in both runs: "all performed correctly". Phone and Bluefy versions were not recorded. Whether he also tried the optional Physical-mode step is not recorded.

## Failed attempts before the passing run

Two laptop runs stopped at the UDP takeover: the car logged the claim but the client heard no HELLO_ACK. Cause was the laptop, not the protocol: wired (.148) and Wi-Fi (.244) on one subnet both answered the car's ARP (`arp_ignore=0`), and a capture showed two `is-at` replies, so replies could go to the wrong interface. The first explanation, harness latency, was wrong; `stand.py` still carries its harmless HELLO retry. For the passing runs `net.ipv4.conf.all.arp_ignore=1` was set temporarily (one ARP reply per request, 5/5 first-contact HELLOs across fresh boots) and restored to 0 afterwards. Their logs were overwritten by the passing run. Home SSID redacted.
