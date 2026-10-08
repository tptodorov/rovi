#!/usr/bin/env bash
# The sim car must rejoin the ROS graph after the router goes away and comes back.
# Needs docker and `cargo build --features ros` in sim/simcar.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
docker build -q -t rovi-ros-sim "$here" >/dev/null
router() {
  docker rm -f rovi-router >/dev/null 2>&1 || true
  docker run -d --rm --name rovi-router --net host rovi-ros-sim \
    bash -c 'source /opt/ros/jazzy/setup.bash && ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
}
nodes() { docker exec rovi-router bash -c 'source /opt/ros/jazzy/setup.bash && ros2 node list 2>/dev/null' || true; }
wait_for_node() {  # up to $1 seconds
  for _ in $(seq "$1"); do nodes | grep -q '^/rovi$' && return 0; sleep 1; done
  return 1
}
trap 'kill $car 2>/dev/null || true; docker rm -f rovi-router >/dev/null 2>&1 || true' EXIT
router; sleep 3
"$here/simcar/target/debug/simcar" 17779 >/tmp/rovi-reconnect.log 2>&1 & car=$!
wait_for_node 15 || { echo "FAIL: the car never joined the graph"; exit 1; }
for cycle in 1 2 3; do  # repeated drops must not leak or wedge the adapter
  echo "joined; killing the router (cycle $cycle)"
  docker rm -f rovi-router >/dev/null; sleep 2
  router; sleep 3
  if ! wait_for_node 40; then echo "FAIL: the car did not rejoin after router restart $cycle"; tail -5 /tmp/rovi-reconnect.log; exit 1; fi
done
echo "reconnect: ok"
