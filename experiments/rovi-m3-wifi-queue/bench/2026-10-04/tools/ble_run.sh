#!/usr/bin/env bash
# usage: ble_run.sh <serial-log> <run-name> <steps...>
set -uo pipefail
log=$1; run=$2; shift 2
cd /tmp/rovi-m3
L=$(wc -l < "$log")
nix-shell -p 'python3.withPackages(p:[p.bleak])' --run "python3 ble_bench.py $*" > "$run.jsonl" 2>&1
sleep 1.5
tail -n +$((L+1)) "$log" > "serial-$run.log"
cat "$run.jsonl"; echo ---; cat "serial-$run.log"
