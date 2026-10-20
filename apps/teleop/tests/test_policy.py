import unittest
from apps.client.wire import Capabilities, Status
from apps.teleop.protocol import ARM, ClientError
from apps.teleop.policy import Policy

CAPS=Capabilities(1,.03,.1275,1,1,3,1000,10000,5000)
ZERO=(0.,0.,0.)

class PolicyTests(unittest.TestCase):
    def armed(self,caps=CAPS):
        p=Policy(caps);self.assertEqual(p.input(b'rw',0),'arm')
        self.assertEqual(p.vector(0),ZERO)
        p.observe(Status('Armed',None,None),ARM)
        return p

    def test_no_implicit_arm_and_new_direction_after_arm(self):
        p=Policy(CAPS);p.input(b'w',0);self.assertEqual(p.vector(0),ZERO)
        p=self.armed();self.assertEqual(p.vector(1),ZERO)
        p.input(b'w',2);self.assertEqual(p.vector(2),(.1,0,0))

    def test_all_six_signs_and_latest_only(self):
        p=self.armed()
        for key,want in [(b'w',(.1,0,0)),(b's',(-.1,0,0)),(b'a',(0,.1,0)),(b'd',(0,-.1,0)),(b'q',(0,0,.3)),(b'e',(0,0,-.3))]:
            p.input(key,10);self.assertEqual(p.vector(10),want)
        p.input(b'wsae',11);self.assertEqual(p.vector(11),(0,0,-.3))

    def test_zero_and_smaller_limits(self):
        p=self.armed(CAPS._replace(max_vx=.05,max_vy=0,max_wz=.2))
        for key,want in [(b'w',(.05,0,0)),(b'a',ZERO),(b'q',(0,0,.2))]:
            p.input(key,10);self.assertEqual(p.vector(10),want)

    def test_expiry_boundary_repeat_and_unrelated_keys(self):
        p=self.armed();p.input(b'w',100)
        p.input(b'x',349);self.assertNotEqual(p.vector(349),ZERO)
        self.assertEqual(p.vector(350),ZERO)
        p.input(b'w',400);p.input(b'w',649)
        self.assertNotEqual(p.vector(898),ZERO);self.assertEqual(p.vector(899),ZERO)

    def test_stop_and_quit_dominate_batch_and_clear_arm(self):
        for keys,want in [(b'w r','stop'),(b'wr\x03','quit'),(b'wr\x1b','quit'),(b'wr\x04','quit')]:
            p=self.armed();p.input(b'w',1)
            self.assertEqual(p.input(keys,2),want)
            self.assertFalse(p.armed);self.assertEqual(p.vector(3),ZERO)
            p.input(b'w',4);self.assertEqual(p.vector(4),ZERO)

    def test_reported_stop_and_errors_cancel_motion(self):
        p=self.armed();p.input(b'w',1)
        p.observe(Status('Stopped','LeaseExpired',None),None)
        p.observe(Status('Armed',None,None),None)
        self.assertFalse(p.armed);self.assertEqual(p.vector(2),ZERO)
        p.observe(Status('Armed',None,'Invalid'),ARM)
        self.assertFalse(p.armed)

    def test_loop_gap_boundary_fails_without_catchup(self):
        p=self.armed();p.input(b'w',0);p.iteration(0);p.iteration(199)
        with self.assertRaises(ClientError):p.iteration(399)
        self.assertEqual(p.vector(399),ZERO);self.assertFalse(p.armed)
