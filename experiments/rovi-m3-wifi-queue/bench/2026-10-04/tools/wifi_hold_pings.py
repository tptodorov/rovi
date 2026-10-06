# Queue movement then keep the Wi-Fi lease alive with pings. argv: moves pings interval
import sys, time
from client import Client
moves, pings, interval = sys.argv[1].split(","), int(sys.argv[2]), float(sys.argv[3])
c = Client("192.168.4.1")
for m in filter(None, moves):
    c.send(c.frame(m))
for _ in range(pings):
    time.sleep(interval); c.send(c.frame("ping"))
c.close()
