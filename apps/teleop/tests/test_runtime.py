import unittest
from unittest.mock import Mock
from apps.teleop.protocol import Session, LinkError, frame, parse, ACK, ARM, STOP, BYE, ECHO, STATUS, SETPOINT
from apps.teleop.runtime import Client
from apps.teleop.tests.test_protocol import CAPS

class Clock:
    def __init__(self):self.now=0
    def __call__(self):return self.now
    def wait(self,readers,timeout):self.now+=max(1,timeout);return readers[:1]

class Sock:
    def __init__(self,fail=()):self.sent=[];self.fail=fail
    def sendto(self,data,peer):
        self.sent.append(parse(data))
        if parse(data)[0] in self.fail:raise OSError('injected send failure')
    def recvfrom(self,n):raise BlockingIOError

class RuntimeTests(unittest.TestCase):
    def client(self,fail=()):
        clock=Clock();sock=Sock(fail);output=[]
        c=Client(('127.0.0.1',7777),sock,0,output.append,clock=clock,wait=clock.wait)
        q=parse(c.session.start(0))[2];c.receive(frame(ACK,42,q,CAPS),1)
        return c,clock,sock,output

    def test_cleanup_attempts_every_send_despite_failures(self):
        for fail in [(SETPOINT,),(STOP,),(SETPOINT,STOP),(BYE,)]:
            c,clock,sock,out=self.client(fail)
            self.assertFalse(c.cleanup())
            self.assertEqual([p[0] for p in sock.sent],[SETPOINT,STOP,BYE])
            self.assertLessEqual(clock.now,500)
            self.assertIn('unconfirmed',' '.join(out))

    def test_stop_before_arm_reply_cannot_reenable_policy(self):
        c,_,_,_=self.client();q=parse(c.session.control(ARM,2))[2]
        c.receive(frame(STATUS,42,0,b'\x03\x02\x00'),3)
        c.receive(frame(STATUS,42,q,b'\x02\x00\x00'),4)
        self.assertFalse(c.policy.armed)

    def test_echo_does_not_refresh_status_age(self):
        c,_,_,_=self.client();q=parse(c.session.control(ARM,10))[2]
        c.receive(frame(STATUS,42,q,b'\x02\x00\x00'),11)
        _,_,seq,payload=parse(c.session.echo(20));c.receive(frame(ECHO,42,seq,payload),30)
        text=c.snapshot(100)
        self.assertIn('status_age_ms=89',text);self.assertIn('echo_age_ms=70',text)
        self.assertIn('last_reported=',text);self.assertNotIn('measured',text)

    def test_wrong_endpoint_cannot_arm(self):
        c,_,sock,_=self.client();q=parse(c.session.control(ARM,2))[2]
        queue=[(frame(STATUS,42,q,b'\x02\x00\x00'),('127.0.0.2',7777))]
        def receive(_n):
            if queue:return queue.pop(0)
            raise BlockingIOError
        sock.recvfrom=receive;c.drain_socket();self.assertFalse(c.policy.armed)

    def test_cleanup_only_accepts_matching_stop_ack(self):
        c,clock,sock,out=self.client()
        def receive(_n):
            stop=next(p for p in sock.sent if p[0]==STOP)
            return frame(STATUS,42,stop[2],b'\x03\x01\x00'),c.peer
        sock.recvfrom=receive
        self.assertTrue(c.cleanup());self.assertLessEqual(clock.now,500)
        self.assertIn('STOP acknowledged',' '.join(out))

    def test_unconfirmed_handshake_still_attempts_session_zero_cleanup(self):
        clock=Clock();sock=Sock();output=[]
        c=Client(('127.0.0.1',7777),sock,0,output.append,clock=clock,wait=clock.wait)
        self.assertFalse(c.cleanup())
        self.assertEqual([p[0] for p in sock.sent],[SETPOINT,STOP,BYE])
        self.assertTrue(all(p[1]==0 for p in sock.sent))
        self.assertLessEqual(clock.now,500)

    def test_one_alert_per_outage_until_echo_confirms_recovery(self):
        c,clock,_,out=self.client()
        c.alert=Mock(return_value=False);c.cleanup=Mock(return_value=True)
        c.keys=lambda:b'rw' # Buffered keys during waits cannot request controls.
        attempts=[]
        def attempt():
            attempts.append(clock.now)
            if len(attempts)==3:
                q=parse(c.session.start(clock.now))[2]
                c.receive(frame(ACK,43,q,CAPS),clock.now)
                # A new claim alone cannot clear the outage alert latch.
                self.assertTrue(c.recovering)
                packet=c.session.echo(clock.now)
                c.receive(packet,clock.now)
                self.assertFalse(c.recovering)
                self.assertFalse(c.policy.armed)
            if len(attempts)<4:raise LinkError('injected outage')
        c.run_session=attempt
        c.run()
        self.assertEqual(c.alert.call_count,2)
        self.assertTrue(all(b-a>=1000 for a,b in zip(attempts,attempts[1:])))
        self.assertEqual(sum('\a' in s for s in out),2)

    def test_socket_failures_are_recoverable_link_errors(self):
        c,_,sock,_=self.client(fail=(ARM,))
        with self.assertRaises(LinkError):c.send(c.session.control(ARM,2))
        sock.recvfrom=Mock(side_effect=OSError('network unavailable'))
        with self.assertRaises(LinkError):c.drain_socket()
