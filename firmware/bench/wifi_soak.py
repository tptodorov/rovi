#!/usr/bin/env python3
"""Observe #23 without claiming the car or changing the laptop's connection."""
import argparse
import datetime
import json
import math
from pathlib import Path
import secrets
import socket
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from apps.teleop.__main__ import target
from apps.teleop.protocol import ECHO, frame, parse


def timestamp():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def echo(sock, sequence):
    payload = secrets.token_bytes(16)
    packet = frame(ECHO, 0, sequence, payload)
    started = time.monotonic()
    deadline = started + .15
    try:
        sock.settimeout(.15)
        sock.send(packet)
        while time.monotonic() < deadline:
            sock.settimeout(max(.001, deadline - time.monotonic()))
            data = sock.recv(1500)
            try:
                reply = parse(data)
            except ValueError:
                continue
            if reply == (ECHO, 0, sequence, payload):
                return {'ok': True, 'rtt_ms': round((time.monotonic()-started)*1000, 3)}
        return {'ok': False, 'error': 'no matching Echo'}
    except OSError as exc:
        return {'ok': False, 'error': type(exc).__name__, 'errno': exc.errno}


def command(args):
    try:
        r = subprocess.run(args, capture_output=True, text=True, timeout=5)
        return {'command': args, 'exit': r.returncode, 'stdout': r.stdout, 'stderr': r.stderr}
    except (OSError, subprocess.TimeoutExpired) as exc:
        return {'command': args, 'error': type(exc).__name__}


def snapshot(output, label, interface, host):
    """Commands are read-only; bounded pauses between probes appear in timestamps."""
    data = {'started': timestamp(), 'commands': [command(args) for args in [
        ['nmcli', '-g', 'GENERAL.STATE,GENERAL.CONNECTION,IP4.ADDRESS,IP4.GATEWAY', 'device', 'show', interface],
        ['iw', 'dev', interface, 'link'],
        ['iw', 'dev', interface, 'get', 'power_save'],
        ['ip', '-j', 'route', 'get', host],
        ['ping', '-n', '-c', '3', '-W', '1', host],
    ]], 'finished': timestamp()}
    (output / f'{label}.json').write_text(json.dumps(data, indent=2)+'\n')


def observe(peer, seconds, output, interface, collect=snapshot):
    output.mkdir(parents=True, exist_ok=False)
    started = timestamp()
    sent = lost = transitions = 0
    previous = None
    complete = False
    try:
        collect(output, 'start', interface, peer[0])
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock, (output/'echo.jsonl').open('w') as log:
            sock.connect(peer)  # The kernel filters replies from other endpoints.
            t0 = time.monotonic()
            deadline = t0 + seconds
            while time.monotonic() < deadline:
                sent += 1
                row = {'at': timestamp(), 'elapsed_s': round(time.monotonic()-t0, 3),
                       'sequence': sent, **echo(sock, sent)}
                lost += not row['ok']
                log.write(json.dumps(row)+'\n'); log.flush()
                if row['ok'] != previous:
                    transitions += 1
                    previous = row['ok']
                    print(json.dumps({'event': 'link', **row}), flush=True)
                    collect(output, f'transition-{transitions:03d}', interface, peer[0])
                time.sleep(min(.2, max(0, deadline-time.monotonic())))
            complete = True
    finally:
        collect(output, 'end', interface, peer[0])
        journal = command(['journalctl', '--no-pager', '--output=short-iso-precise',
                           '--since', started, '--lines=2000',
                           '-u', 'NetworkManager', '-u', 'wpa_supplicant'])
        (output/'journal.json').write_text(json.dumps(journal, indent=2)+'\n')
        summary = {'started': started, 'finished': timestamp(), 'target': list(peer),
                   'requested_seconds': seconds, 'interface': interface, 'complete': complete,
                   'sent': sent, 'lost': lost, 'link_transitions': transitions,
                   'verdict': ('loss observed' if lost else 'no loss observed') if complete else 'incomplete',
                   'reassociation_requested': False, 'journal_limit_lines': 2000}
        (output/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
        print(json.dumps(summary), flush=True)
    return int(lost > 0)


def duration(value):
    n = float(value)
    if not math.isfinite(n) or not 0 < n <= 3600:
        raise argparse.ArgumentTypeError('seconds must be finite and in (0, 3600]')
    return n


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--target', type=target, required=True, metavar='IPV4:PORT')
    p.add_argument('--interface', required=True)
    p.add_argument('--seconds', type=duration, default=900)
    p.add_argument('--output', type=Path, required=True, help='new evidence directory')
    args = p.parse_args()
    try:
        return observe(args.target, args.seconds, args.output, args.interface)
    except KeyboardInterrupt:
        return 130
    except OSError as exc:
        p.exit(2, f'{exc}\n')


if __name__ == '__main__':
    raise SystemExit(main())
