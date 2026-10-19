#!/usr/bin/env bash
# Announced ROS board probes: stand/wheels clear, motor bank awake, ROS firmware,
# Rovi Wi-Fi joined and live serial capture. Default probe: six gentle 3 s drives;
# alternate probes use their own announced procedure.
# ROVI_CARLOG=<live-car.log> bash firmware/bench/ros_stops.sh [probe.py]
# The probe defaults to ros_stop_checks.py; ROVI_TARGET is passed through for UDP probes.
set -euo pipefail
probe=${1:-ros_stop_checks.py}
[[ $probe =~ ^[a-z_]+\.py$ ]]
: "${ROVI_CARLOG:?set the live serial capture}"
test -f "$ROVI_CARLOG"
root=$(cd "$(dirname "$0")/../.." && pwd)
scenarios=$root/apps/simcar/scenarios
log=$(realpath "$ROVI_CARLOG")
port=${ROVI_ROUTER_PORT:-7447}
[[ $port =~ ^[0-9]+$ && $port -ge 1 && $port -le 65535 ]]
name=rovi-ros-stops
if docker container inspect "$name" >/dev/null 2>&1; then
  echo "Container $name already exists; refusing to replace it" >&2
  exit 1
fi
if [ -n "$(ss -H -ltn "sport = :$port")" ]; then
  echo "Router port $port already has a listener" >&2
  exit 1
fi
offset=$(stat -c %s "$log")
docker build -q -t rovi-ros-sim "$scenarios" >/dev/null
docker run -d --rm --name "$name" --net host \
  -e ROVI_ROUTER_PORT="$port" -v "$scenarios:/scenarios:ro" -v "$root/firmware/bench:/bench:ro" -v "$log:$log:ro" \
  rovi-ros-sim bash -c 'set -e
    source /opt/ros/jazzy/setup.bash
    cfg=/opt/ros/jazzy/share/rmw_zenoh_cpp/config
    sed "s#tcp/\\[::\\]:7447#tcp/[::]:$ROVI_ROUTER_PORT#g" "$cfg/DEFAULT_RMW_ZENOH_ROUTER_CONFIG.json5" > /tmp/router.json5
    sed "s#tcp/localhost:7447#tcp/localhost:$ROVI_ROUTER_PORT#g" "$cfg/DEFAULT_RMW_ZENOH_SESSION_CONFIG.json5" > /tmp/session.json5
    export ZENOH_ROUTER_CONFIG_URI=/tmp/router.json5
    ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
trap 'docker rm -f "$name" >/dev/null 2>&1 || true' EXIT
PYTHONPATH="$scenarios" python3 - "$log" "$offset" <<'PYWAIT'
import sys
from carlog import wait, ev
found, seen = wait(sys.argv[1], int(sys.argv[2]), ev('ros', link='up'), timeout=20)
assert found, ('ROS link did not come up', seen[-5:])
PYWAIT
docker exec -e ROVI_CARLOG="$log" -e ROVI_TARGET="${ROVI_TARGET:-}" -e PYTHONPATH=/scenarios \
  -e ZENOH_SESSION_CONFIG_URI=/tmp/session.json5 "$name" \
  bash -c 'set -e; source /opt/ros/jazzy/setup.bash
    python3 "/bench/$1"' _ "$probe"
