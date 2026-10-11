#!/usr/bin/env bash
# One bench run into bench/<name>/: the UDP scenarios against the car, then a consistency check of the car's `ev=` log.
#   sim/bench.sh                                   sim car (build it first: cd sim/simcar && cargo build)
#   ROVI_TARGET=192.168.4.1:7777 ROVI_CARLOG=<serial capture> sim/bench.sh
#                                                  the board: flashed, laptop on its AP, serial logged to that file
# BENCH_NAME names the folder (default sim-<time> or board-<time>); BENCH_CAM=<seconds> records laptop camera frames meanwhile.
set -euo pipefail
cd "$(dirname "$0")/.."
kind=${ROVI_TARGET:+board}; name=${BENCH_NAME:-${kind:-sim}-$(date +%Y%m%d-%H%M%S)}
dir=bench/$name; mkdir -p "$dir"
[ -n "${ROVI_TARGET:-}" ] || export ROVI_CARLOG=${ROVI_CARLOG:-$PWD/$dir/car.log}
[ -z "${BENCH_CAM:-}" ] || ../../scripts/cam.sh "$dir/cam" "$BENCH_CAM" &
status=0
python3 sim/udp_scenarios.py 2>&1 | tee "$dir/scenarios.txt" || status=1
wait
if [ -n "${ROVI_CARLOG:-}" ]; then
  [ "$ROVI_CARLOG" -ef "$dir/car.log" ] || cp "$ROVI_CARLOG" "$dir/car.log"
  python3 sim/carlog.py "$dir/car.log" | tee "$dir/carlog.txt" || status=1
fi
echo "bench run: $dir (exit $status)"
exit $status
