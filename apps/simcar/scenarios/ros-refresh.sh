#!/usr/bin/env bash
# Lost liveliness tokens (UDP does not resend): the car must ask for them again when a sample comes from a
# publisher no token announced. The sim car drops the token events of its first 3 s (ROVI_SIM_IGNORE_TOKENS_S),
# which is when the controller node announces itself. Run twice: with the refresh disabled the car never
# learns the node and never claims; with it the car recovers and claims.
# Needs docker and `cargo build --features ros` in apps/simcar.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
docker build -q -t rovi-ros-sim "$here" >/dev/null
docker rm -f rovi-router >/dev/null 2>&1 || true
docker run -d --rm --name rovi-router --net host -v "$here/ros_controller.py:/ros_controller.py:ro" rovi-ros-sim \
  bash -c 'source /opt/ros/jazzy/setup.bash && ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
trap 'kill $car 2>/dev/null || true; docker rm -f rovi-router >/dev/null 2>&1 || true' EXIT
sleep 3
fail=0
run() {  # run <log> [env...]: car with the token loss, then the controller
  log=$1; shift
  env ROVI_SIM_IGNORE_TOKENS_S=3 "$@" "$here/../target/debug/simcar" 17795 >"$log" 2>&1 & car=$!
  sleep 1
  docker exec -e ROVI_GRAPH_WAIT_S=2 rovi-router bash -c 'source /opt/ros/jazzy/setup.bash && python3 /ros_controller.py' >/dev/null 2>&1
  kill $car 2>/dev/null || true; wait $car 2>/dev/null || true; sleep 2
}
run /tmp/rovi-norefresh.log ROVI_SIM_NO_REFRESH=1
grep -q "ev=claimed source=Ros" /tmp/rovi-norefresh.log && { echo "FAIL: claimed without a refresh (the token loss was not reproduced)"; fail=1; } || echo "ok: without the refresh the car never learns the node (no claim)"
run /tmp/rovi-refresh.log
grep -q "ev=ros link=refresh" /tmp/rovi-refresh.log || { echo "FAIL: no refresh event"; fail=1; }
grep -q "ev=claimed source=Ros" /tmp/rovi-refresh.log && echo "ok: with the refresh the car recovers and claims" || { echo "FAIL: no claim after the refresh"; fail=1; }
[ $fail = 0 ] && echo "ros refresh: ok"
exit $fail
