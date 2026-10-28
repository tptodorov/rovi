#!/usr/bin/env bash
# Network-mode scenarios across ROS, UDP and virtual BLE against the sim car (ADR-0016): UDP and BLE
# take over from a streaming ROS node, the takeover notice, deliberate and accidental hand-back,
# the ROS mode command and release, and /rovi/network. Timelines are in ros_network.py.
# Needs docker and `cargo build --features ros` in apps/simcar. Development evidence only.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
docker build -q -t rovi-ros-sim "$here" >/dev/null
name=rovi-router-net
docker rm -f $name >/dev/null 2>&1 || true
docker run -d --rm --name $name -v "$here/ros_network.py:/ros_network.py:ro" rovi-ros-sim \
  bash -c 'source /opt/ros/jazzy/setup.bash && ros2 run rmw_zenoh_cpp rmw_zenohd' >/dev/null
ip=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $name)
car=
trap 'kill $car 2>/dev/null || true; docker rm -f $name >/dev/null 2>&1 || true' EXIT
sleep 3
fail=0

run() {  # plan, home, host-side python
  local plan=$1 home=$2 host=$3
  kill $car 2>/dev/null || true; sleep 0.5
  ROVI_SIM_HOME=$home ROVI_ZENOH_ROUTER=tcp/$ip:7447 "$here/../target/debug/simcar" 17778 \
    </dev/null >/tmp/rovi-net-$plan.log 2>&1 & car=$!
  sleep 2
  docker exec -e ROVI_PLAN=$plan $name bash -c 'source /opt/ros/jazzy/setup.bash && python3 /ros_network.py' \
    >/tmp/rovi-net-$plan.ros & local node=$!
  sleep 2  # the node's own graph wait: its t=0
  PYTHONPATH="$here" python3 - <<<"$host"
  wait $node
  echo "--- plan $plan: sim car log"; cat /tmp/rovi-net-$plan.log
  echo "--- plan $plan: ROS view"; cat /tmp/rovi-net-$plan.ros
}

udp='
import socket, struct, time
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
def send(kind, session=0, seq=1):
    s.sendto(b"RV\x01" + bytes([kind]) + struct.pack("<II", session, seq), ("127.0.0.1", 17778))
time.sleep(3); send(1); s.settimeout(1); d, _ = s.recvfrom(256)
assert d[3] == 2, d  # HELLO_ACK: UDP took over from ROS
session = struct.unpack_from("<I", d, 4)[0]
time.sleep(1); send(6, session, 2)  # BYE without Home
'
run A 0 "$udp"
log=/tmp/rovi-net-A.log
check() { if ! "$@"; then echo "FAIL: $*"; fail=1; fi; }
check grep -q "ev=preempted source=Ros" $log
check grep -q "ev=claimed source=Udp" $log
check grep -q "status .*state=stopped reason=owner-lost" /tmp/rovi-net-A.ros
check grep -q "ev=mode_cmd source=Ros cmd=CarNetwork result=Ok" $log
check grep -q "ev=net mode=Fallback reason=Requested" $log
check grep -q "network .*mode=fallback" /tmp/rovi-net-A.ros
check python3 - $log <<'PY'
import sys
sys.path.insert(0, sys.argv[0].rsplit("/", 1)[0] if "/" in sys.argv[0] else ".")
ev = [dict(kv.split("=", 1) for kv in l.split()) for l in open(sys.argv[1]) if l.startswith("ev=")]
i = next(k for k, e in enumerate(ev) if e["ev"] == "released" and k > 0 and ev[k - 1].get("ev") != "preempted"
         and any(x["ev"] == "claimed" and x.get("source") == "Udp" for x in ev[:k]))
reclaim = next(e for e in ev[i:] if e["ev"] == "claimed" and e["source"] == "Ros")
moving = next(e for e in ev[i:] if e["ev"] == "state" and e.get("wheels_pm") == "500,500,500,500")
gap = int(moving["at_ms"]) - int(reclaim["at_ms"])
assert gap >= 1500, f"ROS drove {gap} ms after the takeover ended: the latch did not hold"
PY

ble='
import socket, struct, time
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM); car = ("127.0.0.1", 17779)
def ask(req):
    s.sendto(req, car); s.settimeout(1); return s.recvfrom(256)[0]
from wire import cdr
zero = struct.pack("<I", 1) + cdr(0.0)  # seq + CDR TwistStamped at rest
time.sleep(3)
assert ask(b"\x01") == b"\x81\x01"         # advertising in home mode while ROS drives
s.sendto(b"\x02" + zero, car); time.sleep(0.05)  # first write: takes over
assert ask(b"\x03\x03") == b"\x83\x00"     # Home: deliberate hand-back
ask(b"\x06")
time.sleep(3)
assert ask(b"\x01") == b"\x81\x01"
s.sendto(b"\x02" + struct.pack("<I", 2) + cdr(0.0), car); time.sleep(0.05)
ask(b"\x06")                               # dropped without Home
'
run B 1 "$ble"
log=/tmp/rovi-net-B.log
check grep -q "ev=net mode=Home" $log
check grep -c "ev=preempted source=Ros" $log
check python3 - $log <<'PY'
import sys
ev = [dict(kv.split("=", 1) for kv in l.split()) for l in open(sys.argv[1]) if l.startswith("ev=")]
pre = [k for k, e in enumerate(ev) if e["ev"] == "preempted"]
assert len(pre) == 2, pre
first, second = pre
handback = next(e for e in ev[first:second] if e["ev"] == "claimed" and e["source"] == "Ros")
moving = next(e for e in ev[first:second] if e["ev"] == "state" and e.get("wheels_pm") == "500,500,500,500")
assert int(moving["at_ms"]) - int(handback["at_ms"]) < 300, "the Home hand-back did not re-arm the stream"
after = ev[second:]
assert any(e["ev"] == "claimed" and e["source"] == "Ros" for e in after), "the stream did not claim again"
assert not any(e["ev"] == "state" and e.get("wheels_pm") == "500,500,500,500" for e in after), \
    "the stream drove after an accidental BLE drop"
PY
[ $fail = 0 ] && echo "ros network scenarios: ok"
exit $fail
