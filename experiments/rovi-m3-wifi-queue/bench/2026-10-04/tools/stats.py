import json, re, statistics, sys
def pct(v, p):
    v = sorted(v); return v[min(len(v)-1, int(round(p/100*(len(v)-1))))]
def show(name, v):
    print(f"{name}: n={len(v)} median={statistics.median(v):.1f} p95={pct(v,95):.1f} max={max(v):.1f} ms")
jl, serial = sys.argv[1], sys.argv[2]
recs = [json.loads(l) for l in open(jl) if '"rtt_ms"' in l]
cmd = [r for r in recs if r["seq"] > 1]  # seq 1 is the calibration ping
show("ACK RTT (commands)", [r["rtt_ms"] for r in cmd])
st = {}
for r in cmd: s = re.search(r"status=(\w+)", r["response"]).group(1); st[s] = st.get(s, 0) + 1
print("statuses:", st)
q = [int(m[0]) - int(m[1]) for m in re.findall(r"OUT at_ms=(\d+) .*?Apply.*?received_ms: (\d+)", open(serial).read())]
show("board queue latency (Apply)", q)
