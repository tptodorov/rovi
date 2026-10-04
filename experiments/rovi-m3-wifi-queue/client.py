#!/usr/bin/env python3
"""Direct M3 TCP client; standard library only. JSONL is bench evidence, not a verdict."""
import argparse
import json
import socket
import struct
import time


class Client:
    def __init__(self, host):
        self.socket = socket.create_connection((host, 7777), timeout=3)
        self.reader = self.socket.makefile("rb")
        banner = self.reader.readline().decode().strip()
        self.anchor = time.monotonic_ns()
        fields = dict(item.split("=", 1) for item in banner.split()[1:])
        if not banner.startswith("ROVI ") or fields.get("version") != "1":
            self.close()
            raise ValueError(f"unsupported discovery: {banner}")
        self.session = int(fields["session"])
        self.board_ms = int(fields["now_ms"])
        self.age_ms = int(fields["max_age_ms"])
        self.commands = {name: int(code) for code, name in
                         (pair.split("=") for pair in fields["commands"].split(","))}
        self.seq = 0
        print(json.dumps({"discovery": fields}), flush=True)

    def frame(self, command, ttl=None, version=1):
        self.seq += 1
        deadline = self.board_ms + (time.monotonic_ns() - self.anchor) // 1_000_000
        deadline += self.age_ms // 2 if ttl is None else ttl
        code = self.commands[command] if command in self.commands else int(command)
        return struct.pack("<BBQIQ", version, code, self.session, self.seq, max(0, deadline))

    def receive(self, sent, frame):
        response = self.reader.readline().decode().strip()
        if not response:
            raise ConnectionError("session closed; reconnect and send fresh intent")
        received = time.monotonic_ns()
        record = {"session": self.session, "seq": struct.unpack_from("<I", frame, 10)[0],
                  "tx_ns": sent, "ack_ns": received, "rtt_ms": (received - sent) / 1e6,
                  "response": response}
        print(json.dumps(record), flush=True)
        return response

    def send(self, frame):
        sent = time.monotonic_ns()
        self.socket.sendall(frame)
        return self.receive(sent, frame)

    def close(self):
        self.reader.close()
        self.socket.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("commands", nargs="*", default=["forward", "reverse", "stop", "ping"])
    parser.add_argument("--host", default="192.168.4.1")
    parser.add_argument("--repeat", type=int, default=1)
    parser.add_argument("--interval", type=float, default=0.05)
    parser.add_argument("--burst", action="store_true", help="send all frames before reading ACKs")
    parser.add_argument("--ttl", type=int, help="device deadline offset in ms; -1 injects stale input")
    parser.add_argument("--version", type=int, default=1)
    parser.add_argument("--send-delay", type=float, default=0, help="delay after encoding to inject old input")
    parser.add_argument("--hold", type=float, default=0, help="keep socket open, send nothing, observe watchdog")
    parser.add_argument("--reconnect-replay", action="store_true", help="try last frame on a new session")
    args = parser.parse_args()
    if args.repeat < 1 or min(args.interval, args.send_delay, args.hold) < 0:
        parser.error("repeat must be positive; delays must be nonnegative")
    client = Client(args.host)
    pending = []
    frame = None
    try:
        for _ in range(args.repeat):
            for command in args.commands:
                frame = client.frame(command, args.ttl, args.version)
                time.sleep(args.send_delay)
                sent = time.monotonic_ns()
                client.socket.sendall(frame)
                if args.burst:
                    pending.append((sent, frame))
                else:
                    client.receive(sent, frame)
                    time.sleep(args.interval)
        for sent, payload in pending:
            client.receive(sent, payload)
        if args.hold:
            time.sleep(args.hold)
            print(json.dumps({"idle_s": args.hold, "socket_eof": client.reader.read(1) == b""}), flush=True)
        if args.reconnect_replay and frame:
            client.close()
            client = Client(args.host)
            response = client.send(frame)
            if "status=Session" not in response:
                raise AssertionError(f"old session was not rejected: {response}")
    finally:
        client.close()


if __name__ == "__main__":
    main()
