import struct
import unittest
from apps.client.wire import Status
from apps.teleop.protocol import Session, ClientError, frame, parse, HELLO, ACK, ARM, STOP, STATUS, ECHO, BUSY

CAPS = struct.pack("<B5f3H", 1, .03, .1275, 1, 1, 3, 1000, 10000, 5000)

class ProtocolTests(unittest.TestCase):
    def claimed(self):
        s = Session(); request = parse(s.start(0))
        self.assertEqual(s.receive(frame(ACK, 42, request[2], CAPS), 1)[0], 'claimed')
        return s

    def test_handshake_is_single_and_validates_caps(self):
        s = self.claimed()
        self.assertEqual(s.session, 42)
        with self.assertRaises(ClientError): s.start(2)
        for caps in [CAPS[:1]+struct.pack('<f', float('nan'))+CAPS[5:], CAPS[:21]+struct.pack('<3H',199,1000,5000), bytes([2])+CAPS[1:]]:
            s=Session(); seq=parse(s.start(0))[2]
            with self.assertRaises(ClientError): s.receive(frame(ACK,42,seq,caps),1)

    def test_invalid_frames_do_not_refresh_deadline(self):
        s=Session(); seq=parse(s.start(0))[2]
        for data in [b'',b'RV',frame(ACK,42,seq,CAPS)[:-1],frame(ACK,42,seq+1,CAPS),b'ZZ'+frame(ACK,42,seq,CAPS)[2:]]:
            self.assertIsNone(s.receive(data,999))
        with self.assertRaises(ClientError): s.check(1000)
        with self.assertRaises(ValueError): parse(frame(99,0,0))

    def test_busy_and_zero_session_fail(self):
        for kind,session,payload in [(BUSY,1,b''),(ACK,0,CAPS)]:
            s=Session(); seq=parse(s.start(0))[2]
            with self.assertRaises(ClientError): s.receive(frame(kind,session,seq,payload),1)

    def test_control_correlation_stop_cancels_pending_arm(self):
        s=self.claimed(); seq=parse(s.control(ARM,10))[2]
        self.assertIsNone(s.receive(frame(STATUS,41,seq,b'\x02\x00\x00'),11))
        self.assertIsNone(s.receive(frame(STATUS,42,seq+1,b'\x02\x00\x00'),11))
        self.assertEqual(s.receive(frame(STATUS,42,0,b'\x03\x02\x00'),12),('status',Status('Stopped','LeaseExpired',None),None))
        self.assertIsNone(s.receive(frame(STATUS,42,seq,b'\x02\x00\x00'),13))
        seq=parse(s.control(ARM,14))[2]
        self.assertEqual(s.receive(frame(STATUS,42,seq,b'\x02\x00\x00'),15)[2],ARM)

    def test_echo_nonce_session_and_age_are_independent(self):
        s=self.claimed(); seq=parse(s.control(ARM,10))[2]
        s.receive(frame(STATUS,42,seq,b'\x02\x00\x00'),11)
        _,_,q,payload=parse(s.echo(20))
        self.assertIsNone(s.receive(frame(ECHO,42,q,b'bad'),21))
        self.assertIsNone(s.receive(frame(ECHO,43,q,payload),21))
        self.assertEqual(s.receive(frame(ECHO,42,q,payload),22)[0],'echo')
        self.assertEqual(s.status_at,11)
        self.assertEqual(s.echo_at,22)
        s.check(520)
        s.echo(521)
        with self.assertRaises(ClientError):s.check(1021)

    def test_sequence_wrap_and_control_timeout(self):
        s=self.claimed();s.sequence=0xffffffff
        self.assertEqual(parse(s.control(STOP,2))[2],1)
        with self.assertRaises(ClientError):s.check(502)
        s.setpoint_sequence=0xffffffff
        self.assertEqual(parse(s.setpoint((0,0,0)))[2],0)

    def test_bad_status_cannot_enable(self):
        s=self.claimed();q=parse(s.control(ARM,0))[2]
        for b in [b'\x09\x00\x00',b'\x02\x01\x00',b'\x02\x00',b'\x02\x00\x01']:
            self.assertIsNone(s.receive(frame(STATUS,42,q,b),1))
        with self.assertRaises(ClientError):s.check(500)

    def test_invalid_floats_and_window_bounds_are_refused(self):
        for index in range(5):
            for invalid in [float('nan'),float('inf'),float('-inf'),-1.]:
                values=[.03,.1275,1.,1.,3.];values[index]=invalid
                s=Session();q=parse(s.start(0))[2]
                payload=struct.pack('<B5f3H',1,*values,1000,10000,5000)
                with self.subTest(index=index,invalid=invalid), self.assertRaises(ClientError):
                    s.receive(frame(ACK,42,q,payload),1)
        for windows in [(199,10000,5000),(1000,0,5000),(1000,10000,0)]:
            s=Session();q=parse(s.start(0))[2]
            payload=struct.pack('<B5f3H',1,.03,.1275,1,1,3,*windows)
            with self.subTest(windows=windows), self.assertRaises(ClientError):s.receive(frame(ACK,42,q,payload),1)
