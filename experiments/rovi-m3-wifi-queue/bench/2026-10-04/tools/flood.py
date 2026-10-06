# Fill the queue, then flood one kind of non-renewing input every 50 ms; print when the board closes.
import json, sys, time
from client import Client
kind = sys.argv[1]
c = Client("192.168.4.1")
for _ in range(8):
    c.send(c.frame("forward"))
dup = c.frame("ping"); c.send(dup)  # admitted once; later copies are duplicates
start = time.monotonic_ns()
try:
    if kind == "partial":
        c.socket.sendall(c.frame("ping")[:1])
        print(json.dumps({"partial_sent": True}), flush=True)
        r = c.reader.readline()
        print(json.dumps({"closed_after_ms": (time.monotonic_ns() - start) / 1e6, "read": r.decode()}), flush=True)
    else:
        for _ in range(60):
            f = {"invalid": lambda: c.frame("255"), "v2": lambda: c.frame("ping", version=2),
                 "dup": lambda: dup, "expired": lambda: c.frame("forward", ttl=-1),
                 "ping": lambda: c.frame("ping")}[kind]()
            c.send(f)
            time.sleep(0.2 if kind == "ping" else 0.05)
        print(json.dumps({"survived": True}), flush=True)
except (ConnectionError, OSError) as e:
    print(json.dumps({"closed_after_ms": (time.monotonic_ns() - start) / 1e6, "error": type(e).__name__}), flush=True)
