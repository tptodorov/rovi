# Embedded Nix Linux for Raspberry Pi Zero 2 W

Research date: 2026-10-27. Scope: a Pi Zero 2 W with 512MB RAM, a 2GB microSD,
an unidentified CSI Raspberry Pi camera and USB Wi-Fi adapter, running an onboard
ROS 2 marker-following brain that commands the existing ESP32 Rovi.
This is a source review; no image or hardware was tested.

## Recommendation

**Recommended starting point (engineering judgment): minimized NixOS, built on a
larger computer, with `nvmd/nixos-raspberrypi` board support and a narrowly selected
ROS 2 runtime from `lopsided98/nix-ros-overlay`.** Treat fitting the complete system
on 2GB and operating within 512MB as acceptance tests, not established facts.

NixOS already has an [image-based appliance profile](https://github.com/NixOS/nixpkgs/blob/master/nixos/modules/profiles/image-based-appliance.nix).
It imports `minimal.nix`, disables Nix (`nix.enable = false`) and system switching
(`system.switch.enable = false`), makes users immutable, and defaults to networkd.
Its documented model is whole-partition image updates, rather than rebuilding on
the target. The [NixOS manual](https://nixos.org/manual/nixos/stable/#sec-image-based-appliances)
describes appliance images, repart and image update layouts.
**Inference:** this retains NixOS packaging and service configuration while
removing the need for a Nix daemon, evaluator or rebuild workflow on the Pi.
It is a design starting point, not a ready-made Pi camera/ROS appliance image.

[`nvmd/nixos-raspberrypi`](https://github.com/nvmd/nixos-raspberrypi#readme)
explicitly provides `raspberry-pi-02.base`, an `rpi02` installer image, matched
vendor kernel/firmware packages and optimized packages including libcamera.
**Inference:** this is a stronger starting point for this board than porting a
router distribution. Combining its boot support with the appliance profile
still requires configuration and validation; a stock installer is not the
proposed minimized runtime.

## Practical comparison

| Candidate | Verified upstream properties | Assessment for this robot (judgment) |
| --- | --- | --- |
| Minimized NixOS | Appliance profile removes Nix and switching; nvmd explicitly supports Zero 2. Sources above. | Best first Nix route: preserve existing ROS and Pi packaging; measure the resulting image. |
| [Liminix](https://www.liminix.org/doc/#what-is-liminix) | Nix-built OpenWrt-style router/IoT system; BusyBox, musl and s6-rc rather than glibc/systemd. Its documented hardware list covers routers and QEMU, without a Zero 2 entry. | Good embedded Nix example, but camera, board and ROS integration become additional porting work. Not the recommended ROS camera base. |
| [not-os, original](https://github.com/cleverca22/not-os) | Describes itself as experimental; generates kernel, initrd and a 48MB squashfs. Uses NixOS infrastructure and runit. | Useful for a custom appliance, but the quoted base filesystem size says nothing about a complete ROS/camera image. Historical Pi code does not establish Zero 2 camera support. |
| [Alpine](https://alpinelinux.org/about/) | musl/BusyBox distribution with apk and OpenRC; upstream quotes about 130MB for a minimal disk installation. | Small general-purpose alternative, but that is a base size, not ROS/camera sizing. Verify the exact ROS dependencies and Pi camera stack before choosing it. |
| [Buildroot](https://buildroot.org/) | Cross-compiles a toolchain, root filesystem, kernel and bootloader. | Strong option for a tightly selected embedded runtime; expect explicit ROS and camera integration. It does not supply NixOS configuration semantics. |

### Do not confuse the two not-os repositories

The [cleverca22 original](https://github.com/cleverca22/not-os) and
[noteed fork](https://github.com/noteed/not-os) are separate repositories.
Both reported `archived: false` through the GitHub API at research time.
The fork's README explicitly says it removed RPi and iPXE support to make the
system easier to explore. Neither its changes nor any fork's archival status
should be attributed to the original. The original's
[Pi kernel expression](https://github.com/cleverca22/not-os/blob/master/linux-rpi.nix)
is historical board integration, not evidence of a current Zero 2 camera image.

## ROS, camera and storage constraints

The [ROS overlay README](https://github.com/lopsided98/nix-ros-overlay#readme)
lists Jazzy and advertises prebuilt packages for `aarch64-linux` and
`x86_64-linux`. It provides a `ros-core` example and distinguishes its tested
`master` branch from development. This is a packaging route, not a certification
that every desired camera or marker package builds on ARM or meets this RAM limit.
**Recommendation:** pin compatible revisions; include only the chosen middleware,
camera interface, marker detector and ESP32 command bridge. Keep RViz, simulators,
compilers and development tools on the workstation. Build elsewhere; whether a
particular package cross-compiles or needs an ARM builder remains to be checked.

The [official Raspberry Pi camera documentation](https://www.raspberrypi.com/documentation/accessories/camera-software.html)
uses the modern camera stack and marks the legacy stack unsupported; Zero boards
also need the appropriate mini camera connector cable. **Recommendation:** identify
the sensor/module and confirm capture with the chosen kernel, device-tree overlay,
firmware and libcamera before integrating ROS. Identify the USB Wi-Fi chipset too;
its required driver/firmware must be included in the final image.

**No complete-image fit claim is justified yet.** Measure the full runtime
closure and actual partitioned image: boot firmware/kernel/initrd, root filesystem,
camera libraries, ROS middleware, vision dependencies, application, writable data
and free-space reserve. A compressed download or base squashfs size is not the
card requirement. A/B updates need additional partition capacity; the 2GB card
may rule them out. Measure peak RAM and sustained frame latency at the intended
resolution/rate as well. These are proposed acceptance checks, not measured results.

**Fallback recommendation:** if the complete ROS runtime fails either budget,
use a larger card and/or reduce the vision runtime; if ROS on the Pi is optional,
a single camera/marker/control service is another architecture to evaluate.
Nix can still build that appliance on the workstation without installing Nix on
the robot, as demonstrated by the NixOS appliance profile and Liminix's
[off-device build workflow](https://www.liminix.org/doc/#requirements).
