#!/usr/bin/env bash
# Board use needs an announced go, ROS firmware, motor power disconnected,
# Rovi Wi-Fi joined and a live serial capture. Probes never arm; serial checks this.
# ROVI_TARGET=192.168.4.1:7777 ROVI_CARLOG=<live-car.log> bash firmware/bench/ros_readonly.sh
set -euo pipefail
: "${ROVI_TARGET:?set the explicit UDP target}"
: "${ROVI_CARLOG:?set the live serial capture}"
test -f "$ROVI_CARLOG"
root=$(cd "$(dirname "$0")/../.." && pwd)
scenarios=$root/apps/simcar/scenarios
log=$(realpath "$ROVI_CARLOG")
port=${ROVI_ROUTER_PORT:-7447}
[[ $port =~ ^[0-9]+$ && $port -ge 1 && $port -le 65535 ]]
name=rovi-ros-readonly
if docker container inspect "$name" >/dev/null 2>&1; then
  echo "Container $name already exists; refusing to replace it" >&2
  exit 1
fi
if [ -n "$(ss -H -ltn "sport = :$port")" ]; then
  echo "Router port $port already has a listener" >&2
  exit 1
fi
offset=$(stat -c %s "$log")
initial_offset=$offset
docker build -q -t rovi-ros-sim "$scenarios" >/dev/null
docker run -d --rm --name "$name" --net host \
  -e ROVI_ROUTER_PORT="$port" -v "$scenarios:/repo/apps/simcar/scenarios:ro" -v "$root/apps/client:/repo/apps/client:ro" -v "$log:$log:ro" \
  rovi-ros-sim bash -c 'set -e
    source /opt/ros/jazzy/setup.bash
    cfg=/opt/ros/jazzy/share/rmw_zenoh_cpp/config
    sed "s#tcp/\\[::\\]:7447#tcp/[::]:$ROVI_ROUTER_PORT#g" "$cfg/DEFAULT_RMW_ZENOH_ROUTER_CONFIG.json5" > /tmp/router.json5
    sed "s#tcp/localhost:7447#tcp/localhost:$ROVI_ROUTER_PORT#g" "$cfg/DEFAULT_RMW_ZENOH_SESSION_CONFIG.json5" > /tmp/session.json5
    export ZENOH_ROUTER_CONFIG_URI=/tmp/router.json5
    ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
trap 'docker rm -f "$name" >/dev/null 2>&1 || true' EXIT
for stage in initial reconnected; do
  PYTHONPATH="$scenarios" python3 - "$log" "$offset" <<'PY'
import sys
from carlog import wait, ev
found, seen = wait(sys.argv[1], int(sys.argv[2]), ev('ros', link='up'), timeout=20)
assert found, ('ROS link did not come up', seen[-5:])
PY
  echo "ROS read-only probes: $stage"
  docker exec -e ROVI_TARGET="$ROVI_TARGET" -e ROVI_CARLOG="$log" \
    -e ZENOH_SESSION_CONFIG_URI=/tmp/session.json5 "$name" \
    bash -c 'set -e; source /opt/ros/jazzy/setup.bash
      python3 /repo/apps/simcar/scenarios/ros_capabilities.py
      python3 /repo/apps/simcar/scenarios/ros_status.py'
  if [ "$stage" = initial ]; then
    offset=$(stat -c %s "$log")
    docker restart "$name" >/dev/null
  fi
done
PYTHONPATH="$scenarios" python3 - "$log" "$initial_offset" <<'PY'
import sys
from carlog import events, ev
assert not any(ev('state', state='Armed')(event)
               for event in events(sys.argv[1], int(sys.argv[2]))), 'read-only probes armed the car'
PY
echo 'ROS read-only probes: passed before/after router restart; never armed'
