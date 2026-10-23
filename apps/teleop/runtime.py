"""Nonblocking terminal/socket effects around the tested session and input policy."""
from contextlib import contextmanager
import os
import select
import termios
import time
import tty
from .policy import Policy, ZERO
from .protocol import Session, ClientError, LinkError, frame, parse, network_text, ARM, STOP, STATUS, MODE, HOME, CAR_NETWORK
from .audio import beep
from apps.client.wire import Status


def milliseconds():
    return time.monotonic_ns() // 1_000_000


def wait_readable(readers, timeout_ms):
    return select.select(readers, [], [], max(0,timeout_ms)/1000)[0]


@contextmanager
def terminal(input_fd, output_fd):
    if not os.isatty(input_fd):
        raise ClientError('an interactive terminal is required')
    saved = termios.tcgetattr(input_fd)
    flags = {fd: os.get_blocking(fd) for fd in (input_fd,output_fd)}
    try:
        tty.setcbreak(input_fd)
        attrs = termios.tcgetattr(input_fd)
        attrs[3] &= ~termios.ISIG  # Ctrl-C is handled as an input byte through normal cleanup.
        termios.tcsetattr(input_fd,termios.TCSANOW,attrs)
        for fd in flags:
            os.set_blocking(fd,False)
        yield
    finally:
        try:
            termios.tcsetattr(input_fd,termios.TCSANOW,saved)
        finally:
            for fd,blocking in flags.items():
                try:
                    os.set_blocking(fd,blocking)
                except OSError:
                    pass  # A closed terminal still gets all remaining restoration attempts.


def console(fd, message):
    try:
        os.write(fd,(message+'\n').encode())
    except BlockingIOError:
        pass  # A full output pipe must not delay the command loop.


def read_keys(input_fd):
    chunks = []
    for _ in range(16):
        try:
            data = os.read(input_fd,4096)
        except BlockingIOError:
            return b''.join(chunks)
        if not data:
            raise ClientError('terminal closed')
        chunks.append(data)
    raise ClientError('excess buffered terminal input; aborting')


class Client:
    def __init__(self, peer, sock, input_fd, emit, *, clock=milliseconds, wait=wait_readable, alert=beep, mode=None):
        self.peer,self.sock,self.input_fd,self.emit = peer,sock,input_fd,emit
        self.clock,self.wait = clock,wait
        self.session = Session()
        self.policy = None
        self.cleanup_ok = False
        self.alert = alert
        self.connected_once = False
        self.recovering = False
        self.alerted = False
        self.discard_keys = False
        # ADR-0016: Home or Car network, sent again on every claim (a command belongs to its session).
        self.mode = {'home': HOME, 'car': CAR_NETWORK, None: None}[mode]

    def send(self, packet):
        try:
            self.sock.sendto(packet,self.peer)
        except OSError as exc:
            raise LinkError(f'UDP send failed: {exc}') from exc

    def receive(self, packet, now):
        event = self.session.receive(packet,now)
        if not event:
            return
        if event[0] == 'claimed':
            self.policy = Policy(event[1])
            self.discard_keys = True
            self.send(self.session.info(now))
            if self.mode:
                self.send(self.session.control(MODE,now,bytes([self.mode])))
            if self.recovering:
                self.emit(f'Claimed disarmed session={self.session.session} capabilities={dict(event[1]._asdict())}; verifying Echo')
            else:
                self.connected_once = True
                self.emit(f'Connected disarmed session={self.session.session} capabilities={dict(event[1]._asdict())}')
        elif event[0] == 'echo' and self.recovering:
            self.recovering = self.alerted = False
            self.discard_keys = True
            label = 'Reconnected' if self.connected_once else 'Connected'
            self.connected_once = True
            self.emit(f'{label} disarmed session={self.session.session}; press R to arm')
        elif event[0] == 'info':
            self.emit(f'Car network: {network_text(event[2])}')
        elif event[0] == 'status' and event[2] == MODE:
            name = 'home' if self.mode == HOME else 'car-network'
            if event[1].error is None:
                self.emit(f'Mode {name} pending; it applies when this controller releases the car (quit)')
            else:
                self.emit(f'Mode {name} refused: {event[1].error}')
        elif event[0] == 'status':
            self.policy.observe(event[1],event[2])
            if event[2] == STOP:
                confirmed = event[1] == Status('Stopped','Stop',None)
                self.emit('STOP acknowledged' if confirmed else 'STOP unconfirmed/refused')

    def drain_socket(self):
        for _ in range(64):
            try:
                data,peer = self.sock.recvfrom(1500)
            except BlockingIOError:
                return
            except OSError as exc:
                raise LinkError(f'UDP receive failed: {exc}') from exc
            if peer == self.peer:
                self.receive(data,self.clock())
        raise ClientError('too many incoming datagrams; aborting')

    def keys(self):
        return read_keys(self.input_fd)

    def snapshot(self, now):
        s = self.session
        age = lambda at: 'none' if at is None else str(max(0,now-at))
        status = '/'.join(str(x) for x in s.status) if s.status else 'none'
        vector = self.policy.vector(now) if self.policy else ZERO
        return (f'connection={"reconnecting" if self.recovering else "connected"} '
                f'local={"armed" if self.policy and self.policy.armed else "disarmed"} session={s.session} '
                f'last_reported={status} status_age_ms={age(s.status_at)} '
                f'echo_age_ms={age(s.echo_at)} requested_vx/vy/wz={vector}')

    def run(self):
        try:
            while True:
                try:
                    self.run_session()
                    return
                except LinkError as exc:
                    if self.policy:
                        self.policy.clear()
                    self.recovering = True
                    label = 'Connection lost' if self.connected_once else 'Connection unavailable'
                    self.emit(f'{label}: {exc}; motion cleared; reconnecting disarmed')
                    self.cleanup()
                    self.policy = None
                    if self.connected_once and not self.alerted:
                        self.alerted = True
                        if not self.alert():
                            self.emit('\aAudio alert unavailable; terminal bell attempted')
                    retry_at = self.clock()+1000
                    while self.clock() < retry_at:
                        ready = self.wait([self.input_fd],min(20,retry_at-self.clock()))
                        if self.input_fd in ready:
                            keys = self.keys()
                            if any(k in keys for k in (3,4,27)):
                                return
                    # Carry the sequence across attempts to reject delayed old ACKs.
                    self.session = Session(self.session.sequence)
        finally:
            self.cleanup_ok = self.cleanup()

    def run_session(self):
        self.send(self.session.start(self.clock()))
        self.emit('R arm | WASD translate | QE rotate | Space STOP | Esc/Ctrl-C quit')
        self.emit('Motion expires after 250 ms without another direction key; key repeat renews it.')
        next_stream = next_echo = next_display = 0
        while True:
            ready = self.wait([self.sock,self.input_fd],20)
            now = self.clock()
            if self.policy:
                self.policy.iteration(now)
            self.session.check(now)
            # Process device stops before any newly received operator intent.
            self.drain_socket()
            if self.input_fd in ready or self.discard_keys:
                keys = self.keys()
                if any(k in keys for k in (3,4,27)):
                    return
                if self.policy and not self.recovering and not self.discard_keys:
                    action = self.policy.input(keys,self.clock())
                    if action == 'quit':
                        return
                    if action == 'stop':
                        self.send(self.session.setpoint(ZERO))
                        self.send(self.session.control(STOP,self.clock()))
                    elif action == 'arm' and not self.session.pending:
                        self.send(self.session.setpoint(ZERO))
                        self.send(self.session.control(ARM,self.clock()))
                self.discard_keys = False
            if not self.policy:
                continue
            now = self.clock()
            self.policy.iteration(now)
            self.session.check(now)
            if now >= next_stream:
                self.drain_socket()
                self.send(self.session.setpoint(self.policy.vector(self.clock())))
                next_stream = self.clock()+50
            if now >= next_echo:
                self.send(self.session.echo(now))
                next_echo = now+250
            if now >= next_display:
                self.emit(self.snapshot(now))
                next_display = now+250

    def cleanup(self):
        if self.policy:
            self.policy.clear()
        deadline = self.clock()+500
        s = self.session
        q = s.next_sequence()
        failures = []
        # Send BYE before waiting, so a missing STOP acknowledgement cannot prevent release attempts.
        packets = [s.setpoint(ZERO),frame(STOP,s.session,q),s.bye()]
        for packet in packets:
            try:
                self.send(packet)
            except (OSError,LinkError) as exc:
                failures.append(str(exc))
        confirmed = False
        while s.session and self.clock() < deadline:
            try:
                if not self.wait([self.sock],min(20,deadline-self.clock())):
                    continue
                data,peer = self.sock.recvfrom(1500)
                if peer != self.peer:
                    continue
                kind,session,seq,payload = parse(data)
                if (kind,session,seq) == (STATUS,s.session,q) and Status.decode(payload) == Status('Stopped','Stop',None):
                    confirmed = True
                    break
            except (BlockingIOError,ValueError):
                continue
            except OSError as exc:
                failures.append(str(exc))
                break
        try:
            self.emit(f'Cleanup: STOP {"acknowledged" if confirmed else "unconfirmed"}; BYE requested (release unconfirmed)'
                      + (f'; send/receive errors={failures}' if failures else ''))
        except OSError:
            pass
        return confirmed and not failures
