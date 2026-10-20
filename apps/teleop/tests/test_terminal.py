import os
import pty
import select
import signal
import socket
import struct
import subprocess
import sys
import termios
import threading
import time
import unittest
from pathlib import Path
from apps.teleop.protocol import parse, frame, HELLO, ACK, ARM, STOP, BYE, STATUS, ECHO, SETPOINT
from apps.teleop.tests.test_protocol import CAPS

ROOT=Path(__file__).resolve().parents[3]

def vector(packet):
    values=struct.unpack_from('<6d',packet[3],20)
    return (values[0],values[1],values[5])

class Peer:
    def __init__(self):
        self.sock=socket.socket(socket.AF_INET,socket.SOCK_DGRAM);self.sock.bind(('127.0.0.1',0));self.sock.settimeout(.02)
        self.address=self.sock.getsockname();self.packets=[];self.drop_echo=False;self.drop_arm=False;self.stop_before_arm=False
        self.alive=True;self.thread=threading.Thread(target=self.serve,daemon=True);self.thread.start()
    def serve(self):
        while self.alive:
            try:data,peer=self.sock.recvfrom(1500)
            except socket.timeout:continue
            except OSError:return
            try:p=parse(data)
            except ValueError:continue
            self.packets.append(p);kind,session,q,payload=p
            reply=None
            if kind==HELLO:reply=frame(ACK,42,q,CAPS)
            if kind==ARM and not self.drop_arm:
                if self.stop_before_arm:self.sock.sendto(frame(STATUS,42,0,b'\x03\x02\x00'),peer)
                reply=frame(STATUS,42,q,b'\x02\x00\x00')
            if kind==STOP:reply=frame(STATUS,42,q,b'\x03\x01\x00')
            if kind==ECHO and not self.drop_echo:reply=data
            if reply:self.sock.sendto(reply,peer)
    def close(self):
        self.alive=False;self.thread.join(timeout=1);self.sock.close()

class Terminal:
    def __init__(self,peer):
        self.master,self.slave=pty.openpty();self.before=termios.tcgetattr(self.slave);self.output=b''
        self.proc=subprocess.Popen([sys.executable,'-u','-m','apps.teleop','--target',f'{peer[0]}:{peer[1]}'],cwd=ROOT,stdin=self.slave,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    def read(self):
        if select.select([self.proc.stdout],[],[],.02)[0]:
            data=os.read(self.proc.stdout.fileno(),65536)
            self.output+=data
    def expect(self,text,timeout=3):
        end=time.monotonic()+timeout
        while text.encode() not in self.output and time.monotonic()<end:self.read()
        if text.encode() not in self.output:raise AssertionError(f'missing {text}: {self.output[-1500:]!r}')
    def key(self,keys):os.write(self.master,keys)
    def finish(self,keys=b'\x04'):
        if keys:self.key(keys)
        self.proc.wait(timeout=3)
        while select.select([self.proc.stdout],[],[],0)[0]:
            data=os.read(self.proc.stdout.fileno(),65536)
            if not data:break
            self.output+=data
        return self.proc.returncode
    def close(self):
        if self.proc.poll() is None:
            self.proc.kill();self.proc.wait(timeout=2)
        self.proc.stdout.close()
        if self.master is not None:os.close(self.master)
        os.close(self.slave)

class TerminalTests(unittest.TestCase):
    def setUp(self):
        self.peer=Peer();self.tty=Terminal(self.peer.address);self.tty.expect('Connected disarmed')
    def tearDown(self):self.tty.close();self.peer.close()
    def test_real_cli_start_arm_expiry_space_rearm_and_restore(self):
        t=self.tty;p=self.peer
        t.key(b'w');time.sleep(.1)
        self.assertFalse(any(x[0]==ARM or (x[0]==SETPOINT and any(vector(x))) for x in p.packets))
        t.key(b'r');t.expect('local=armed');t.key(b'w');time.sleep(.4)
        sent=[vector(x) for x in p.packets if x[0]==SETPOINT]
        self.assertIn((.1,0,0),sent);self.assertEqual(sent[-1],(0,0,0))
        t.key(b' w');t.expect('STOP acknowledged');start=len(p.packets);t.key(b'w');time.sleep(.1)
        self.assertFalse(any(x[0]==SETPOINT and any(vector(x)) for x in p.packets[start:]))
        t.key(b'r');time.sleep(.1);t.key(b'e');time.sleep(.1)
        self.assertIn((0,0,-.3),[vector(x) for x in p.packets if x[0]==SETPOINT])
        self.assertEqual(t.finish(),0)
        self.assertEqual(termios.tcgetattr(t.slave),t.before)
        self.assertEqual(sum(x[0]==HELLO for x in p.packets),1)
        self.assertEqual(p.packets[-1][0],BYE)
        self.assertIn(b'release unconfirmed',t.output)
    def test_lost_echo_exits_and_restores_without_reclaim(self):
        self.peer.drop_echo=True
        self.tty.proc.wait(timeout=3);self.tty.finish(None)
        self.assertEqual(self.tty.proc.returncode,1)
        self.assertEqual(termios.tcgetattr(self.tty.slave),self.tty.before)
        self.assertEqual(sum(x[0]==HELLO for x in self.peer.packets),1)
        self.assertTrue(any(x[0]==BYE for x in self.peer.packets))
    def test_missing_arm_reply_fails_without_nonzero(self):
        self.peer.drop_arm=True;self.tty.key(b'rw');self.tty.proc.wait(timeout=3);self.tty.finish(None)
        self.assertEqual(self.tty.proc.returncode,1)
        self.assertFalse(any(x[0]==SETPOINT and any(vector(x)) for x in self.peer.packets))
    def test_stop_before_arm_reply_needs_new_operator_arm(self):
        self.peer.stop_before_arm=True;self.tty.key(b'r');time.sleep(.15);self.tty.key(b'w');time.sleep(.15)
        self.assertFalse(any(x[0]==SETPOINT and any(vector(x)) for x in self.peer.packets))
        self.assertEqual(self.tty.finish(b'\x03'),0)
        self.assertEqual(termios.tcgetattr(self.tty.slave),self.tty.before)
    def test_termination_signal_runs_cleanup(self):
        self.tty.proc.send_signal(signal.SIGTERM);self.tty.proc.wait(timeout=3);self.tty.finish(None)
        self.assertEqual(self.tty.proc.returncode,1)
        self.assertEqual(termios.tcgetattr(self.tty.slave),self.tty.before)
        self.assertTrue(any(x[0]==BYE for x in self.peer.packets))
    def test_non_tty_and_invalid_target_never_connect(self):
        start=sum(p[0]==HELLO for p in self.peer.packets)
        for target in [f'{self.peer.address[0]}:{self.peer.address[1]}','host.invalid:7777','127.0.0.1:0']:
            result=subprocess.run([sys.executable,'-m','apps.teleop','--target',target],cwd=ROOT,input=b'rw',capture_output=True,timeout=2)
            self.assertNotEqual(result.returncode,0)
        self.assertEqual(sum(p[0]==HELLO for p in self.peer.packets),start)

    def test_stalled_process_discards_buffered_motion_on_resume(self):
        self.tty.key(b'r');self.tty.expect('local=armed')
        self.tty.proc.send_signal(signal.SIGSTOP);time.sleep(.3)
        start=len(self.peer.packets);self.tty.key(b'w')
        self.tty.proc.send_signal(signal.SIGCONT)
        self.tty.proc.wait(timeout=3);self.tty.finish(None)
        self.assertEqual(self.tty.proc.returncode,1)
        self.assertFalse(any(p[0]==SETPOINT and any(vector(p)) for p in self.peer.packets[start:]))
        self.assertEqual(termios.tcgetattr(self.tty.slave),self.tty.before)

    def test_terminal_hangup_still_requests_stop_and_bye(self):
        os.close(self.tty.master);self.tty.master=None
        self.tty.proc.wait(timeout=3);self.tty.finish(None)
        self.assertEqual(self.tty.proc.returncode,1)
        self.assertTrue(any(p[0]==STOP for p in self.peer.packets))
        self.assertTrue(any(p[0]==BYE for p in self.peer.packets))
