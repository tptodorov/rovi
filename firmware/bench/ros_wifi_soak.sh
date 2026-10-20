#!/usr/bin/env bash
# Idle ROS-router keepalives plus stateless UDP Echo; never claim or arm.
# Existing normal ROS firmware, motor power disconnected, live serial capture.
# ROVI_TARGET=192.168.4.1:7777 ROVI_CARLOG=... bash firmware/bench/ros_wifi_soak.sh IFACE SECONDS OUTPUT
set -euo pipefail
interface=${1:?provide the laptop interface}
seconds=${2:?provide the observation seconds}
output=$(realpath -m "${3:?provide a new evidence directory}")
: "${ROVI_TARGET:?set the explicit UDP target}"
: "${ROVI_CARLOG:?set the live serial capture}"
test -f "$ROVI_CARLOG"
test ! -e "$output"
root=$(cd "$(dirname "$0")/../.." && pwd)
log=$(realpath "$ROVI_CARLOG")
port=${ROVI_ROUTER_PORT:-7447}
[[ $port =~ ^[0-9]+$ && $port -ge 1 && $port -le 65535 ]]
name=rovi-ros-wifi-soak
if docker container inspect "$name" >/dev/null 2>&1; then
  echo "Container $name already exists; refusing to replace it" >&2; exit 2
fi
if [ -n "$(ss -H -ltn "sport = :$port")" ]; then
  echo "Router port $port already has a listener" >&2; exit 2
fi
offset=$(stat -c %s "$log")
docker build -q -t rovi-ros-sim "$root/apps/simcar/scenarios" >/dev/null
docker run -d --rm --name "$name" --net host -e ROVI_ROUTER_PORT="$port" \
  rovi-ros-sim bash -c 'set -e
    source /opt/ros/jazzy/setup.bash
    cfg=/opt/ros/jazzy/share/rmw_zenoh_cpp/config
    sed "s#tcp/\\[::\\]:7447#tcp/[::]:$ROVI_ROUTER_PORT#g" "$cfg/DEFAULT_RMW_ZENOH_ROUTER_CONFIG.json5" > /tmp/router.json5
    export ZENOH_ROUTER_CONFIG_URI=/tmp/router.json5
    ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
finish() {
  rc=$?
  mkdir -p "$output"
  docker logs "$name" > "$output/router.log" 2>&1 || true
  docker cp "$name:/tmp/router.json5" "$output/router.json5" >/dev/null 2>&1 || true
  docker rm -f "$name" >/dev/null 2>&1 || true
  exit "$rc"
}
trap finish EXIT
PYTHONPATH="$root/apps/simcar/scenarios" python3 - "$log" "$offset" <<'PY'
import sys
from carlog import wait, ev
found, seen = wait(sys.argv[1], int(sys.argv[2]), ev('ros', link='up'), timeout=25)
assert found, ('ROS link did not come up', seen[-5:])
PY
echo 'ROS link up; observing idle router session without ROS application publishers'
probe_rc=0
python3 "$root/firmware/bench/wifi_soak.py" --target "$ROVI_TARGET" \
  --interface "$interface" --seconds "$seconds" --output "$output" || probe_rc=$?
PYTHONPATH="$root/apps/simcar/scenarios" python3 - "$log" "$offset" "$output" <<'PY'
import json, sys
from pathlib import Path
from carlog import events
seen=events(sys.argv[1],int(sys.argv[2]))
assert not any(e['ev'] in ('boot','claimed','released','stop','error','dropped')
               or e.get('state')=='Armed' or any(e.get('wheels_pm',[])) for e in seen), seen
links=[e for e in seen if e['ev']=='ros']
assert any(e.get('link')=='up' for e in links), 'missing ROS session evidence'
Path(sys.argv[3],'ros-link.json').write_text(json.dumps({'events':links},indent=2)+'\n')
print('Serial ownership/motor/fault checks passed; ROS link transitions recorded')
PY
test "$(docker inspect -f '{{.State.Running}}' "$name")" = true
exit "$probe_rc"
