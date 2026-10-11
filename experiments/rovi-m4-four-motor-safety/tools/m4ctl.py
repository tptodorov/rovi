#!/usr/bin/env python3
"""Drive the M4 bench firmware over its serial port and keep the car log.

Each STEP is one argument:
  "en" | "stby" | "zero brake" | "hang high" ...   sent once, as is (see src/command.rs)
  "w FL FR RR RL for MS"                            streamed at 20 Hz for MS ms (renews the watchdog)
  "wait MS"                                         send nothing for MS ms (lets the watchdog fire)

Every line the car prints is written unchanged to LOG (and echoed), so docs/LOGGING.md parsers
work on it. Standard library only (termios), Linux. Untried on the board.

  tools/m4ctl.py /dev/ttyACM0 bench/2026-10-20-t4/car.log "en" "w 0.3 0 0 0 for 2000" "wait 500"
"""

import argparse
import os
import sys
import termios
import threading
import time

RATE_HZ = 20


def steps(args):
    """Parse step arguments into (kind, line, ms) tuples; raise ValueError on a bad one."""
    out = []
    for a in args:
        words = a.split()
        if not words:
            raise ValueError("empty step")
        if words[0] == "wait" and len(words) == 2:
            out.append(("wait", None, int(words[1])))
        elif words[0] == "w" and len(words) == 7 and words[5] == "for":
            out.append(("stream", " ".join(words[:5]), int(words[6])))
        else:
            out.append(("send", a, 0))
    return out


def open_port(path):
    fd = os.open(path, os.O_RDWR | os.O_NOCTTY)
    attrs = termios.tcgetattr(fd)
    attrs[0] = 0  # iflag
    attrs[1] = 0  # oflag
    attrs[2] = termios.CS8 | termios.CREAD | termios.CLOCAL  # cflag, no HUPCL
    attrs[3] = 0  # lflag: raw
    attrs[4] = attrs[5] = termios.B115200
    attrs[6][termios.VMIN] = 1
    attrs[6][termios.VTIME] = 0
    termios.tcsetattr(fd, termios.TCSANOW, attrs)
    return fd


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("port")
    ap.add_argument("log")
    ap.add_argument("step", nargs="+")
    ap.add_argument("--tail-ms", type=int, default=1000, help="keep logging this long after the last step")
    a = ap.parse_args()
    plan = steps(a.step)

    fd = open_port(a.port)
    os.makedirs(os.path.dirname(os.path.abspath(a.log)), exist_ok=True)
    log = open(a.log, "ab", buffering=0)
    done = threading.Event()

    def reader():
        while not done.is_set():
            data = os.read(fd, 256)
            log.write(data)
            sys.stdout.buffer.write(data)
            sys.stdout.flush()

    threading.Thread(target=reader, daemon=True).start()

    def send(line):
        os.write(fd, line.encode() + b"\n")

    for kind, line, ms in plan:
        if kind == "send":
            send(line)
        elif kind == "wait":
            time.sleep(ms / 1000)
        else:
            end = time.monotonic() + ms / 1000
            while time.monotonic() < end:
                send(line)
                time.sleep(1 / RATE_HZ)
    time.sleep(a.tail_ms / 1000)
    done.set()


if __name__ == "__main__":
    main()
