import socket, time, struct
for i in range(5):
    t0 = time.monotonic_ns()
    try:
        s = socket.create_connection(("192.168.4.1", 7777), timeout=3)
    except OSError as e:
        print(i, "connect error", e); time.sleep(0.5); continue
    t1 = time.monotonic_ns()
    r = s.makefile("rb"); line = r.readline(); t2 = time.monotonic_ns()
    f = dict(x.split("=",1) for x in line.decode().split()[1:])
    # ping with generous deadline to measure board rx vs banner now_ms
    fr = struct.pack("<BBQIQ", 1, 3, int(f["session"]), 1, int(f["now_ms"]) + 240)
    t3 = time.monotonic_ns(); s.sendall(fr); ack = r.readline().decode().strip(); t4 = time.monotonic_ns()
    rx = int(ack.split("rx_ms=")[1].split()[0])
    print(i, f"connect={(t1-t0)/1e6:.1f}ms banner_wait={(t2-t1)/1e6:.1f}ms rtt={(t4-t3)/1e6:.1f}ms board_rx-now_ms={rx-int(f['now_ms'])}ms client_elapsed={(t3-t2)/1e6:.1f}ms", ack)
    r.close(); s.close(); time.sleep(float(__import__("sys").argv[1]))
