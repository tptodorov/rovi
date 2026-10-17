#!/usr/bin/env bash
# ROS 2 scenarios against the sim car. A stock `ros2 topic pub` on /cmd_vel alone claims, arms and
# drives (ADR-0011). Then a controller node: its stream arms, an intruder node is ignored, a STOP stays
# latched while it streams, an arm resumes. Needs docker and `cargo build --features ros` in apps/simcar.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
docker build -q -t rovi-ros-sim "$here" >/dev/null
# Own container name and docker network, so a bench router on the host does not collide.
name=rovi-router-sim
docker rm -f $name >/dev/null 2>&1 || true
docker run -d --rm --name $name -v "$here/ros_controller.py:/ros_controller.py:ro" -v "$here/ros_latency.py:/ros_latency.py:ro" rovi-ros-sim \
  bash -c 'source /opt/ros/jazzy/setup.bash && ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
ip=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $name)
trap 'kill $car 2>/dev/null || true; docker rm -f $name >/dev/null 2>&1 || true' EXIT
sleep 3
ROVI_ZENOH_ROUTER=tcp/$ip:7447 "$here/../target/debug/simcar" 17778 >/tmp/rovi-sim.log 2>&1 & car=$!
sleep 2
echo "--- stock tool: ros2 topic pub on /cmd_vel only"
docker exec $name bash -c 'source /opt/ros/jazzy/setup.bash && timeout 4 ros2 topic pub -r 20 /cmd_vel geometry_msgs/msg/TwistStamped "{twist: {linear: {x: 0.6}}}" >/dev/null' || true
sleep 7  # lease expiry and reclaim window: the car is released
stock=$(cat /tmp/rovi-sim.log)
docker exec $name bash -c 'source /opt/ros/jazzy/setup.bash && python3 /ros_controller.py' | tee /tmp/rovi-status.log
echo "--- ROS round trip (100 probes)"
docker exec -e ROVI_N=100 $name bash -c 'source /opt/ros/jazzy/setup.bash && python3 /ros_latency.py' | tee /tmp/rovi-ros-latency.log
echo "--- sim car log"; cat /tmp/rovi-sim.log
fail=0
check() { if ! "$@"; then echo "FAIL: $*"; fail=1; fi; }
check grep -q "wheels_pm=300,300,300,300" <<<"$stock"  # the stock tool drove
check grep -q "ev=released" <<<"$stock"
check grep -q "ev=claimed source=Ros" /tmp/rovi-sim.log
check grep -q "wheels_pm=500,500,500,500" /tmp/rovi-sim.log
check grep -q "ev=stop reason=Stop" /tmp/rovi-sim.log
check grep -q "ev=stop reason=OutOfLimits" /tmp/rovi-sim.log
check grep -q "state=armed" /tmp/rovi-status.log
check grep -q "state=stopped" /tmp/rovi-status.log
check python3 - /tmp/rovi-status.log <<'PY'
import sys

with open(sys.argv[1]) as log:
    status = [(float(t), state) for t, state in (line.split(maxsplit=1) for line in log)]
assert any(7.3 <= t < 8.9 and "state=stopped" in s for t, s in status), status
assert not any(7.3 <= t < 8.9 and "state=armed" in s for t, s in status), status
assert any(9.1 <= t < 10.5 and "state=armed" in s for t, s in status), status
parsed = []
for t, text in status:
    tokens = text.split()
    fields = dict(token.split('=', 1) for token in tokens)
    assert len(tokens) == 4 and fields.keys() == {'state', 'reason', 'error', 'session'}, text
    if fields['state'] == 'open':
        assert fields == dict(state='open', reason='none', error='none', session='0'), fields
    parsed.append((t, fields))
assert any(3.2 <= t < 4.4 and f['error'] == 'not-ready' for t, f in parsed), parsed
assert all(f['error'] == 'not-ready' for t, f in parsed if 3.2 <= t < 4.4), parsed
assert any(4.4 <= t < 5.6 and f['error'] == 'none' for t, f in parsed), parsed
assert any(6.3 <= t < 7 and f['reason'] == 'lease-expired' for t, f in parsed), parsed
assert any(7.3 <= t < 8.9 and f['reason'] == 'out-of-limits' for t, f in parsed), parsed
assert any(t >= 15.8 and f['state'] == 'open' for t, f in parsed), parsed
PY
if grep -q "wheels_pm=-" /tmp/rovi-sim.log; then echo "FAIL: the intruder node drove the car"; fail=1; fi
# After the STOP the car stays stopped while the stream goes on, and drives again only after the arm.
after=$(sed -n '/ev=stop reason=Stop/,$p' /tmp/rovi-sim.log)
check grep -q "ev=state state=Armed wheels_pm=500" <<<"$after"
first_armed=$(grep -n "ev=state state=Armed" <<<"$after" | head -1 | cut -d: -f1)
stops=$(head -n "${first_armed:-1}" <<<"$after" | grep -c "wheels_pm=500" || true)
[ "$stops" -le 1 ] || { echo "FAIL: the stream undid the stop"; fail=1; }
[ $fail = 0 ] && echo "ros scenarios: ok"
exit $fail
