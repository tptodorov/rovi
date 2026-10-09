#!/usr/bin/env bash
# First M8 risk item: is a zenoh-nostd node visible in the ROS 2 graph?
# Needs docker and the zenoh-nostd fork built with `cargo build --example z_ros_node --features std,log`.
# Usage: sim/graph-check.sh /path/to/z_ros_node
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
docker build -q -t rovi-ros-sim "$here" >/dev/null
docker rm -f rovi-router >/dev/null 2>&1 || true
docker run -d --rm --name rovi-router --net host rovi-ros-sim \
  bash -c 'source /opt/ros/jazzy/setup.bash && ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
trap 'docker rm -f rovi-router >/dev/null 2>&1 || true' EXIT
sleep 3
RUST_LOG=info timeout 30 "$1" >/tmp/rovi-node.log 2>&1 &
sleep 3
docker exec rovi-router bash -c 'source /opt/ros/jazzy/setup.bash
  ros2 node list
  ros2 topic info -v /cmd_vel
  timeout 6 ros2 topic pub -r 5 /cmd_vel geometry_msgs/msg/TwistStamped "{twist: {linear: {x: 0.1}}}" >/dev/null'
echo "--- node log"; tail -5 /tmp/rovi-node.log
