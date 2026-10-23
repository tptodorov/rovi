#!/usr/bin/env bash
# The transport conformance suite (conformance.py) against one sim car in home mode, from inside
# the ROS container with host networking so UDP, virtual BLE and ROS reach the same car.
# Needs docker and `cargo build --features ros` in apps/simcar. Development evidence only.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
docker build -q -t rovi-ros-sim "$here" >/dev/null
name=rovi-router-conformance port=17790 router=17460 lease=600
docker rm -f $name >/dev/null 2>&1 || true
log=$(mktemp /tmp/rovi-conformance.XXXX)
car=
trap 'kill $car 2>/dev/null || true; docker rm -f $name >/dev/null 2>&1 || true; rm -f "$log"' EXIT
# Router on its own port (as firmware/bench/ros_stops.sh does), so a bench router is not disturbed.
docker run -d --rm --name $name --net host -e ROVI_ROUTER_PORT=$router \
  -v "$here:/repo/apps/simcar/scenarios:ro" -v "$here/../../client:/repo/apps/client:ro" -v "$log:$log:ro" rovi-ros-sim bash -c 'set -e
    source /opt/ros/jazzy/setup.bash
    cfg=/opt/ros/jazzy/share/rmw_zenoh_cpp/config
    sed "s#tcp/\\[::\\]:7447#tcp/[::]:$ROVI_ROUTER_PORT#g" "$cfg/DEFAULT_RMW_ZENOH_ROUTER_CONFIG.json5" > /tmp/router.json5
    sed "s#tcp/localhost:7447#tcp/localhost:$ROVI_ROUTER_PORT#g" "$cfg/DEFAULT_RMW_ZENOH_SESSION_CONFIG.json5" > /tmp/session.json5
    export ZENOH_ROUTER_CONFIG_URI=/tmp/router.json5
    ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
sleep 3
ROVI_SIM_HOME=1 ROVI_ZENOH_ROUTER=tcp/127.0.0.1:$router "$here/../target/debug/simcar" $port $lease 2000 1000 \
  </dev/null >"$log" 2>&1 & car=$!
for _ in $(seq 100); do grep -q "ev=ros link=up" "$log" && break; sleep 0.2; done
grep -q "ev=ros link=up" "$log" || { echo "FAIL: the sim car never reached the router"; cat "$log"; exit 1; }
status=0
docker exec -e ROVI_CARLOG="$log" -e ROVI_PORT=$port -e ROVI_LEASE_MS=$lease \
  -e ZENOH_SESSION_CONFIG_URI=/tmp/session.json5 $name \
  bash -c 'source /opt/ros/jazzy/setup.bash && cd /repo/apps/simcar/scenarios && python3 conformance.py' || status=$?
if [ $status != 0 ]; then echo "--- sim car log (tail)"; tail -40 "$log"; fi
exit $status
