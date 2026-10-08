#!/usr/bin/env bash
# ROS 2 scenario against the sim car: claim by first /cmd_vel, arm from a second publisher of the
# same node, an intruder node ignored, stop. Needs docker and `cargo build --bin simcar --features ros`.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
docker build -q -t rovi-ros-sim "$here" >/dev/null
docker rm -f rovi-router >/dev/null 2>&1 || true
docker run -d --rm --name rovi-router --net host -v "$here/ros_controller.py:/ros_controller.py:ro" rovi-ros-sim \
  bash -c 'source /opt/ros/jazzy/setup.bash && ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
trap 'kill $car 2>/dev/null || true; docker rm -f rovi-router >/dev/null 2>&1 || true' EXIT
sleep 3
"$here/../target/debug/simcar" 17778 >/tmp/rovi-sim.log 2>&1 & car=$!
sleep 2
docker exec rovi-router bash -c 'source /opt/ros/jazzy/setup.bash && python3 /ros_controller.py' | tee /tmp/rovi-status.log
echo "--- sim car log"; cat /tmp/rovi-sim.log
fail=0
check() { if ! "$@"; then echo "FAIL: $*"; fail=1; fi; }
check grep -q "claimed Ros" /tmp/rovi-sim.log
check grep -q "wheels \[0.5, 0.5, 0.5, 0.5\]" /tmp/rovi-sim.log
check grep -q "stop Stop" /tmp/rovi-sim.log
check grep -q "state=armed" /tmp/rovi-status.log
check grep -q "state=stopped" /tmp/rovi-status.log
check ! grep -q "wheels \[-" /tmp/rovi-sim.log   # the intruder node never drove the car
[ $fail = 0 ] && echo "ros scenarios: ok"
exit $fail
