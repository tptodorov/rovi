#!/usr/bin/env bash
# Motor batch: announced go, stand/wheels clear, motor bank awake, ROS firmware,
# Rovi Wi-Fi joined and live serial capture. Two gentle 3 s drives (vx=0.1):
#   KILLED: router killed while driving -> OwnerLost stop + release, no lease stop.
#   FROZEN: router paused while driving -> lease stop, release (stopped-owner
#           window or session end) and link-down within the 10 s router lease
#           (<=35 s); the paused router is then killed.
# The publisher runs inside the router container, so it dies with it and never
# resumes. Each new router must bring a fresh link-up with no claim or arming.
# ROVI_CARLOG=<live-car.log> bash firmware/bench/ros_router_loss.sh
set -euo pipefail
: "${ROVI_CARLOG:?set the live serial capture}"
test -f "$ROVI_CARLOG"
root=$(cd "$(dirname "$0")/../.." && pwd)
scenarios=$root/apps/simcar/scenarios
log=$(realpath "$ROVI_CARLOG")
port=${ROVI_ROUTER_PORT:-7447}
[[ $port =~ ^[0-9]+$ && $port -ge 1 && $port -le 65535 ]]
name=rovi-ros-loss
if docker container inspect "$name" >/dev/null 2>&1; then
  echo "Container $name already exists; refusing to replace it" >&2
  exit 1
fi
if [ -n "$(ss -H -ltn "sport = :$port")" ]; then
  echo "Router port $port already has a listener" >&2
  exit 1
fi
docker build -q -t rovi-ros-sim "$scenarios" >/dev/null
trap 'docker rm -f "$name" >/dev/null 2>&1 || true' EXIT

router() {
  docker run -d --rm --name "$name" --net host -e ROVI_ROUTER_PORT="$port" rovi-ros-sim bash -c 'set -e
    source /opt/ros/jazzy/setup.bash
    cfg=/opt/ros/jazzy/share/rmw_zenoh_cpp/config
    sed -e "s#tcp/\\[::\\]:7447#tcp/[::]:$ROVI_ROUTER_PORT#g" -e "s#lease: 60000#lease: 10000#" \
      -e "s#keep_alive: 2,#keep_alive: 4,#" "$cfg/DEFAULT_RMW_ZENOH_ROUTER_CONFIG.json5" > /tmp/router.json5
    sed "s#tcp/localhost:7447#tcp/localhost:$ROVI_ROUTER_PORT#g" "$cfg/DEFAULT_RMW_ZENOH_SESSION_CONFIG.json5" > /tmp/session.json5
    export ZENOH_ROUTER_CONFIG_URI=/tmp/router.json5
    ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
}

# expect <offset> <timeout> <event> [k=v...]: a fresh matching serial event.
# reject <offset> <event> [k=v...]: no such event since offset.
serial() {
  PYTHONPATH="$scenarios" python3 - "$log" "$@" <<'PY'
import sys
import carlog
path, mode, offset = sys.argv[1], sys.argv[2], int(sys.argv[3])
rest = sys.argv[4:]
timeout = float(rest.pop(0)) if mode == "expect" else 0
name, fields = rest[0], dict(kv.split("=", 1) for kv in rest[1:])
pred = carlog.ev(name, **fields)
if mode == "expect":
    found, seen = carlog.wait(path, offset, pred, timeout=timeout)
    assert found, (name, fields, [e.get("ev") for e in seen[-8:]])
else:
    bad = [e for e in carlog.events(path, offset) if pred(e)]
    assert not bad, ("unexpected", bad)
PY
}
expect() { serial expect "$@"; }
reject() { serial reject "$@"; }
size() { stat -c %s "$log"; }

fresh_router() {
  local at
  at=$(size)
  router
  expect "$at" 20 ros link=up
  echo "Router up; car linked"
}

drive() {
  local at
  at=$(size)
  echo "Gentle forward vx=0.1 starts in 5 s"
  sleep 5
  docker exec -d "$name" bash -c "source /opt/ros/jazzy/setup.bash
    export ZENOH_SESSION_CONFIG_URI=/tmp/session.json5
    ros2 topic pub -r 20 /cmd_vel geometry_msgs/msg/TwistStamped '{twist: {linear: {x: 0.1}}}' >/dev/null 2>&1"
  expect "$at" 10 claimed source=Ros
  expect "$at" 2 state state=Armed
  sleep 3
}

check_new_router_idle() {
  local at
  at=$(size)
  fresh_router
  sleep 3
  reject "$at" claimed
  reject "$at" state state=Armed
  local status
  status=$(docker exec "$name" bash -c 'source /opt/ros/jazzy/setup.bash
    export ZENOH_SESSION_CONFIG_URI=/tmp/session.json5
    timeout 10 ros2 topic echo --once /rovi/status std_msgs/msg/String')
  echo "$status" | grep 'data:'
  grep -q 'state=open reason=none error=none session=0' <<<"$status"
  echo "New router: car open, not claimed or armed"
}

fresh_router

echo
echo "=== KILLED: router killed while driving ==="
drive
at=$(size)
echo "Kill router: wheels must stop at once"
docker kill "$name" >/dev/null
expect "$at" 3 stop reason=OwnerLost
expect "$at" 3 released
expect "$at" 3 ros link=down
reject "$at" stop reason=LeaseExpired
docker rm -f "$name" >/dev/null 2>&1 || true
echo "KILLED: OwnerLost + release; wheel observation required"
check_new_router_idle

echo
echo "=== FROZEN: router frozen while driving ==="
drive
at=$(size)
echo "Freeze router: wheels must stop after about 1 s (lease)"
docker pause "$name" >/dev/null
expect "$at" 3 stop reason=LeaseExpired
echo "Lease stop; waiting for the car to drop the frozen router (up to 35 s)"
expect "$at" 35 released
expect "$at" 35 ros link=down
reject "$at" stop reason=OwnerLost
reject "$at" ros link=up
docker rm -f "$name" >/dev/null
echo "FROZEN: lease stop + release; wheel observation required"
check_new_router_idle

docker rm -f "$name" >/dev/null
echo
echo "ROS router-loss checks: passed; released"
