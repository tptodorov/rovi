"""Run with python3 -m apps.teleop --target IPV4:PORT from the repository root."""
import argparse
import asyncio
import ipaddress
import os
import re
import signal
import socket
import sys
import termios
from .protocol import ClientError
from .runtime import Client, console, terminal


def target(value):
    try:
        host,port = value.rsplit(':',1)
        peer = (str(ipaddress.IPv4Address(host)),int(port))
        if not 1 <= peer[1] <= 65535:
            raise ValueError
        return peer
    except ValueError:
        raise argparse.ArgumentTypeError('target must be an IPv4 address and port in 1..65535') from None


def interrupted(signum, _frame):
    raise ClientError(f'termination signal {signum}')


def ble_address(value):
    if value=='Rovi M8' or re.fullmatch(r'(?:[0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}',value):
        return value
    raise argparse.ArgumentTypeError('BLE target must be a MAC address')


def main(argv=None):
    parser = argparse.ArgumentParser(prog='python3 -m apps.teleop',description='Gentle UDP or BLE keyboard teleoperation. Reconnects disarmed; press R to arm.')
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--target',type=target,metavar='IPV4:PORT')
    mode.add_argument('--ble',type=ble_address,nargs='?',const='Rovi M8',metavar='MAC',help='BLE mode; discover one Rovi M8 or pin its MAC')
    args = parser.parse_args(argv)
    previous = {sig:signal.getsignal(sig) for sig in (signal.SIGTERM,signal.SIGHUP)}
    try:
        for sig in previous:
            signal.signal(sig,interrupted)
        with terminal(sys.stdin.fileno(),sys.stdout.fileno()):
            emit = lambda message:console(sys.stdout.fileno(),message)
            if args.ble is not None:
                from .ble import BleClient
                client = BleClient(args.ble,sys.stdin.fileno(),emit)
                asyncio.run(client.run())
            else:
                with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as sock:
                    sock.setblocking(False)
                    client = Client(args.target,sock,sys.stdin.fileno(),emit)
                    client.run()
            return 0 if client.cleanup_ok else 1
    except (ClientError,OSError,termios.error) as exc:
        os.write(2,f'teleop: {exc}\n'.encode())
        return 1
    except KeyboardInterrupt:
        return 130
    finally:
        for sig,handler in previous.items():
            signal.signal(sig,handler)



if __name__ == '__main__':
    raise SystemExit(main())
