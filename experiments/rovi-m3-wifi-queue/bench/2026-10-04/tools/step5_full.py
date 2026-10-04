from client import Client
c = Client("192.168.4.1")
frames = [c.frame("forward") for _ in range(100)]
import time
sent = time.monotonic_ns()
c.socket.sendall(b"".join(frames))
for frame in frames:
    c.receive(sent, frame)
c.send(c.frame("stop"))
c.close()
