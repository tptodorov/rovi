#!/usr/bin/env bash
# How long does the car take to notice that its router is gone? The car's read timeout is the router's
# lease: rmw_zenohd announces 60000 ms (and a keep-alive every 30 s) by default, so the car needs about
# 60 s. The router is frozen (`docker pause`), not removed: a closed UDP port makes the host answer with an
# ICMP error that a loopback socket reports at once, and the board's stack ignores. Usage: sim/ros-router-gone.sh [lease_ms]   (default 10000; try 60000 for the default router).
# Needs docker and `cargo build --features ros` in sim/simcar. Over UDP, since TCP closes at once.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
lease=${1:-10000}
docker build -q -t rovi-ros-sim "$here" >/dev/null
docker rm -f rovi-router >/dev/null 2>&1 || true
docker run -d --rm --name rovi-router --net host -e LEASE_MS="$lease" rovi-ros-sim bash -c '
  source /opt/ros/jazzy/setup.bash
  sed -e "s#\"tcp/\\[::\\]:7447\"#\"tcp/[::]:7447\", \"udp/0.0.0.0:7448\"#" \
      -e "s#lease: 60000#lease: $LEASE_MS#" -e "s#keep_alive: 2,#keep_alive: 4,#" \
      /opt/ros/jazzy/share/rmw_zenoh_cpp/config/DEFAULT_RMW_ZENOH_ROUTER_CONFIG.json5 > /tmp/router.json5
  ZENOH_ROUTER_CONFIG_URI=/tmp/router.json5 ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
trap 'kill $car 2>/dev/null || true; docker rm -f rovi-router >/dev/null 2>&1 || true' EXIT
sleep 3
ROVI_ZENOH_ROUTER=udp/127.0.0.1:7448 "$here/simcar/target/debug/simcar" 17796 >/tmp/rovi-gone.log 2>&1 & car=$!
for _ in $(seq 20); do grep -q "ev=ros link=up" /tmp/rovi-gone.log && break; sleep 0.5; done
grep -q "ev=ros link=up" /tmp/rovi-gone.log || { echo "FAIL: the car never connected"; exit 1; }
sleep 2
docker pause rovi-router >/dev/null
t0=$(date +%s.%N)
for _ in $(seq 400); do grep -q "ros session ended" /tmp/rovi-gone.log && break; sleep 0.5; done
grep -q "ros session ended" /tmp/rovi-gone.log || { echo "FAIL: not noticed within 200 s"; exit 1; }
took=$(python3 -c "import time; print(f'{time.time() - $t0:.1f}')")
echo "router lease ${lease} ms: the car noticed the router was gone after ${took} s"
python3 -c "import sys; sys.exit(0 if $took <= 3 * $lease / 1000 else 1)" && echo "router gone: ok" || { echo "FAIL: slower than 3 leases"; exit 1; }
