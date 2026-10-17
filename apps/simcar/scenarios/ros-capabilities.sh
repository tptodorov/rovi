#!/usr/bin/env bash
# Capabilities fallback: late Jazzy subscriber, exact UDP parity, no claim by a reader,
# immediate claim/release publication, shared status/errors and router reconnect. Sim car only; no board access.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
name=rovi-capabilities-sim
log=${ROVI_CAPABILITIES_LOG:-/tmp/rovi-capabilities-sim.log}
: >"$log"
docker build -q -t rovi-ros-sim "$here" >/dev/null
# Host networking lets the ROS observer use the host's UDP simulator despite bridge firewall
# rules. A dedicated loopback router port keeps this separate from a bench router on 7447.
docker run -d --rm --name "$name" --net host -v "$here:/scenarios:ro" -v "$log:$log:ro" rovi-ros-sim \
  bash -c 'set -e
    source /opt/ros/jazzy/setup.bash
    cfg=/opt/ros/jazzy/share/rmw_zenoh_cpp/config
    sed "s#tcp/\\[::\\]:7447#tcp/127.0.0.1:17447#g" "$cfg/DEFAULT_RMW_ZENOH_ROUTER_CONFIG.json5" > /tmp/router.json5
    sed "s#tcp/localhost:7447#tcp/127.0.0.1:17447#g" "$cfg/DEFAULT_RMW_ZENOH_SESSION_CONFIG.json5" > /tmp/session.json5
    ZENOH_ROUTER_CONFIG_URI=/tmp/router.json5 ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
car=
trap 'if [ -n "$car" ]; then kill "$car" 2>/dev/null || true; wait "$car" 2>/dev/null || true; fi; docker rm -f "$name" >/dev/null 2>&1 || true' EXIT
sleep 3
ROVI_ZENOH_ROUTER=tcp/127.0.0.1:17447 "$here/../target/debug/simcar" 17794 >"$log" 2>&1 & car=$!
wait_link() {
  for _ in $(seq 100); do
    [ "$(grep -c 'ev=ros link=up' "$log" || true)" -ge "$1" ] && return
    sleep 0.1
  done
  cat "$log"; echo 'FAIL: ROS link did not come up'; exit 1
}
probe() {
  docker exec -e ROVI_TARGET=127.0.0.1:17794 -e ROVI_CARLOG="$log" \
    -e ZENOH_SESSION_CONFIG_URI=/tmp/session.json5 "$name" \
    bash -c 'source /opt/ros/jazzy/setup.bash && python3 /scenarios/ros_capabilities.py && python3 /scenarios/ros_status.py'
}
wait_link 1
sleep 2 # subscriber starts after initial publication
probe
docker restart "$name" >/dev/null
wait_link 2
probe
if grep -q 'state=Armed' "$log"; then echo 'FAIL: capabilities check armed the car'; exit 1; fi
echo 'ros capabilities/status: ok (volatile fallback, including reconnect)'
