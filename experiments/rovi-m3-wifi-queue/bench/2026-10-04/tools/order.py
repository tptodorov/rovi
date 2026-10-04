import re, sys
txt = open(sys.argv[1]).read().splitlines()
rx = [(m["src"], int(m["seq"]), int(m["order"]), m["st"], int(m["rx"]), m["cmd"]) for l in txt
      if (m := re.match(r"RX src=(?P<src>\w+) session=\d+ seq=(?P<seq>\d+) cmd=(?P<cmd>\w+) rx_ms=(?P<rx>\d+) .* order=(?P<order>\d+) depth=\d+ status=(?P<st>\w+)", l))]
for src in ("Wifi", "Ble"):
    seqs = [r[1] for r in rx if r[0] == src]
    print(src, "rx count", len(seqs), "seq strictly increasing:", all(a < b for a, b in zip(seqs, seqs[1:])),
          "rx_ms span", min(r[4] for r in rx if r[0]==src), "-", max(r[4] for r in rx if r[0]==src))
orders = [r[2] for r in rx if r[2]]
print("admission order strictly increasing across sources:", all(a < b for a, b in zip(orders, orders[1:])))
applied = [int(o) for o in re.findall(r"Apply\(Entry .*?order: (\d+)", "\n".join(txt))]
print("Apply count", len(applied), "Apply order strictly increasing (FIFO):", all(a < b for a, b in zip(applied, applied[1:])))
srcs = re.findall(r"Apply\(Entry \{ input: Input \{ source: (\w+)", "\n".join(txt))
print("Apply by source:", {s: srcs.count(s) for s in set(srcs)})
