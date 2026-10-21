import socket
import subprocess
import tempfile
import time
import unittest
from apps.simcar.scenarios.carlog import events
from apps.teleop.tests.test_terminal import Terminal, ROOT

BIN=ROOT/'apps/simcar/target/debug/simcar'

def signs(wheels):return tuple((x>0)-(x<0) for x in wheels)

class SimulatorTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(BIN.exists(),'build apps/simcar before running integration tests')
        with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as reservation:
            reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
        self.port=port
        self.log=tempfile.NamedTemporaryFile()
        self.car=subprocess.Popen([BIN,str(port),'1000','10000','5000'],stdout=self.log,stderr=self.log)
        self.tty=None
        end=time.monotonic()+2
        while not any(e.get('ev')=='boot' for e in events(self.log.name)):
            self.assertLess(time.monotonic(),end,'simulator failed to boot');time.sleep(.01)
        self.tty=Terminal(('127.0.0.1',port));self.tty.expect('Connected disarmed')
    def tearDown(self):
        if self.tty:self.tty.close()
        if self.car.poll() is None:self.car.terminate();self.car.wait(timeout=2)
        self.log.close()
    def wait_state(self,predicate,offset=0):
        end=time.monotonic()+2
        while True:
            found=[e for e in events(self.log.name)[offset:] if e.get('ev')=='state' and predicate(e)]
            if found:return found[-1]
            self.assertLess(time.monotonic(),end,self.log.read().decode(errors='replace'))
            time.sleep(.01)
    def test_six_directions_expiry_stop_latch_and_cleanup(self):
        t=self.tty;t.key(b'w');time.sleep(.1)
        self.assertFalse(any(any(e.get('wheels_pm',[])) for e in events(self.log.name)))
        t.key(b'r');t.expect('local=armed')
        patterns=[(b'w',(1,1,1,1)),(b's',(-1,-1,-1,-1)),(b'a',(-1,1,-1,1)),(b'd',(1,-1,1,-1)),(b'q',(-1,1,1,-1)),(b'e',(1,-1,-1,1))]
        for key,want in patterns:
            start=len(events(self.log.name));t.key(key)
            self.wait_state(lambda e:signs(e.get('wheels_pm',[]))==want,start)
            self.wait_state(lambda e:e.get('wheels_pm')==[0,0,0,0],start)
        t.key(b' w');t.expect('STOP acknowledged');start=len(events(self.log.name));t.key(b'w');time.sleep(.15)
        self.assertFalse(any(any(e.get('wheels_pm',[])) for e in events(self.log.name)[start:]))
        t.key(b'r');time.sleep(.1);start=len(events(self.log.name));t.key(b'w')
        self.wait_state(lambda e:signs(e.get('wheels_pm',[]))==(1,1,1,1),start)
        self.assertEqual(t.finish(),0)
        # Release is logged by UDP before the control loop publishes its next state.
        self.wait_state(lambda e:e.get('state')=='Open' and e.get('wheels_pm')==[0,0,0,0],start)
        end=time.monotonic()+1
        while not any(e.get('ev')=='released' for e in events(self.log.name)):
            self.assertLess(time.monotonic(),end);time.sleep(.01)
        seen=events(self.log.name)
        self.assertEqual(sum(e.get('ev')=='claimed' for e in seen),1)
        self.assertEqual([e for e in seen if e.get('ev')=='state'][-1].get('wheels_pm'),[0,0,0,0])
    def test_restarted_simulator_reconnects_disarmed_until_new_arm_and_direction(self):
        self.tty.key(b'r');self.tty.expect('local=armed');self.tty.key(b'w')
        self.wait_state(lambda e:any(e.get('wheels_pm',[])))
        self.car.terminate();self.car.wait(timeout=2)
        self.tty.expect('Connection lost:')
        self.assertIsNone(self.tty.proc.poll())
        self.tty.key(b'rw')
        start=len(events(self.log.name))
        self.car=subprocess.Popen([BIN,str(self.port),'1000','10000','5000'],stdout=self.log,stderr=self.log)
        self.tty.expect('Reconnected disarmed',timeout=4)
        self.tty.key(b'w');time.sleep(.15)
        self.assertFalse(any(any(e.get('wheels_pm',[])) for e in events(self.log.name)[start:]))
        self.tty.key(b'r');time.sleep(.15);self.tty.key(b'w')
        self.wait_state(lambda e:any(e.get('wheels_pm',[])),start)
        self.assertEqual(self.tty.finish(),0)
        self.assertIn(b'release unconfirmed',self.tty.output)
