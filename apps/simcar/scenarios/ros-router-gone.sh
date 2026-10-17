#!/usr/bin/env bash
# How long does the car take to notice that its router is gone, and to rejoin once it is back? The router
# is frozen (`docker pause`), not removed: a closed UDP port makes the host answer with an ICMP error that a
# loopback socket reports at once, and the board's stack ignores. The router runs on its own docker network
# (not --net host), so this can run next to a bench router.
# Usage: apps/simcar/scenarios/ros-router-gone.sh [lease_ms] [udp|tcp]   (default 10000 tcp; 60000 is the rmw_zenohd default; `60000 udp` shows zenoh#2849).
# Needs docker and `cargo build --features ros` in apps/simcar.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
lease=${1:-10000}
proto=${2:-tcp}
name=rovi-router-gone
log=/tmp/rovi-gone.log
docker build -q -t rovi-ros-sim "$here" >/dev/null
docker rm -f $name >/dev/null 2>&1 || true
docker run -d --rm --name $name -e LEASE_MS="$lease" rovi-ros-sim bash -c '
  source /opt/ros/jazzy/setup.bash
  sed -e "s#\"tcp/\\[::\\]:7447\"#\"tcp/[::]:7447\", \"udp/0.0.0.0:7448\"#" \
      -e "s#lease: 60000#lease: $LEASE_MS#" -e "s#keep_alive: 2,#keep_alive: 4,#" \
      /opt/ros/jazzy/share/rmw_zenoh_cpp/config/DEFAULT_RMW_ZENOH_ROUTER_CONFIG.json5 > /tmp/router.json5
  ZENOH_ROUTER_CONFIG_URI=/tmp/router.json5 ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
ip=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $name)
trap 'kill $car 2>/dev/null || true; docker rm -f $name >/dev/null 2>&1 || true' EXIT
sleep 3
ROVI_ZENOH_ROUTER=$proto/$ip:$([ "$proto" = tcp ] && echo 7447 || echo 7448) "$here/../target/debug/simcar" 17796 >$log 2>&1 & car=$!
count() { grep -c "$1" $log || true; }
for _ in $(seq 40); do [ "$(count 'ev=ros link=up')" -gt 0 ] && break; sleep 0.5; done
[ "$(count 'ev=ros link=up')" -gt 0 ] || { echo "FAIL: the car never connected"; exit 1; }
sleep 2
docker pause $name >/dev/null
t0=$(date +%s.%N)
for _ in $(seq 400); do [ "$(count 'ros session ended')" -gt 0 ] && break; sleep 0.5; done
[ "$(count 'ros session ended')" -gt 0 ] || { echo "FAIL: not noticed within 200 s"; exit 1; }
noticed=$(python3 -c "import time; print(f'{time.time() - $t0:.1f}')")
docker unpause $name >/dev/null
t1=$(date +%s.%N)
for _ in $(seq 240); do [ "$(count 'ev=ros link=up')" -gt 1 ] && break; sleep 0.5; done
back=$(python3 -c "import time; print(f'{time.time() - $t1:.1f}')")
echo "$proto, router lease ${lease} ms: noticed after ${noticed} s; back ${back} s after the router resumed"
python3 -c "import sys; sys.exit(0 if $noticed <= 3 * $lease / 1000 else 1)" || { echo "FAIL: noticed slower than 3 leases"; exit 1; }
[ "$(count 'ev=ros link=up')" -gt 1 ] || { echo "FAIL: not back within 120 s"; exit 1; }
echo "router gone: ok"
