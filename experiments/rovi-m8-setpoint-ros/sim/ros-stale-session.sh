#!/usr/bin/env bash
# The car drops its UDP session silently and rejoins with the same zenoh id while the router still
# holds the old session. It must come back. Needs docker and `cargo build --features ros` in sim/simcar.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
docker build -q -t rovi-ros-sim "$here" >/dev/null
docker rm -f rovi-router >/dev/null 2>&1 || true
# The default router listens on TCP only; add a UDP listener.
docker run -d --rm --name rovi-router --net host rovi-ros-sim bash -c '
  source /opt/ros/jazzy/setup.bash
  sed "s#\"tcp/\\[::\\]:7447\"#\"tcp/[::]:7447\", \"udp/0.0.0.0:7448\"#" \
    /opt/ros/jazzy/share/rmw_zenoh_cpp/config/DEFAULT_RMW_ZENOH_ROUTER_CONFIG.json5 > /tmp/router.json5
  ZENOH_ROUTER_CONFIG_URI=/tmp/router.json5 ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
trap 'kill $car 2>/dev/null || true; docker rm -f rovi-router >/dev/null 2>&1 || true' EXIT
sleep 3
ROVI_ZENOH_ROUTER=udp/127.0.0.1:7448 ROVI_SIM_ABANDON_AFTER_S=8 "$here/simcar/target/debug/simcar" 17780 >/tmp/rovi-stale.log 2>&1 & car=$!
nodes() { docker exec rovi-router bash -c 'source /opt/ros/jazzy/setup.bash && ros2 node list 2>/dev/null' || true; }
for _ in $(seq 15); do nodes | grep -q '^/rovi$' && break; sleep 1; done
nodes | grep -q '^/rovi$' || { echo "FAIL: the car never joined over UDP"; tail -5 /tmp/rovi-stale.log; exit 1; }
echo "joined over UDP; the car abandons its session after 8 s"
sleep 14   # abandon at 8 s, rejoin attempts from 10 s
for _ in $(seq 60); do
  if grep -q "ros session ended" /tmp/rovi-stale.log && [ "$(grep -c 'up, zid' /tmp/rovi-stale.log)" -ge 2 ] && nodes | grep -q '^/rovi$'; then
    echo "stale session: ok"; exit 0
  fi
  sleep 1
done
echo "FAIL: the car did not rejoin"; tail -8 /tmp/rovi-stale.log; exit 1
