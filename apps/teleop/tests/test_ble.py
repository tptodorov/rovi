import asyncio
import os
import struct
import signal
import subprocess
import sys
import termios
import time
import unittest
from types import SimpleNamespace
from unittest.mock import AsyncMock, Mock, patch
from apps.teleop.ble import BleClient, BleakError, NAME, SERVICE, SETPOINT, CONTROL, CAPS, STATUS, ECHO, NETWORK, discover
from apps.teleop.protocol import ClientError, LinkError
from apps.teleop.tests.test_protocol import CAPS as CAPS_VALUE
from apps.teleop.tests.test_terminal import Terminal, ROOT


def vector(data):
    values=struct.unpack_from('<6d',data,24)
    return values[0],values[1],values[5]


class Peer:
    def __init__(self):
        self.is_connected=False;self.mtu_size=247;self.status=b'\x01\x00\x00'
        self.address='AA:00:00:00:00:01'
        self.callbacks={};self.writes=[];self.disconnected=False
        self.drop_echo=False;self.refuse_arm=False;self.fail_writes=False;self.hang_writes=False
        self.arm_hook=None;self.caps=CAPS_VALUE
        self.write_delay=self.read_delay=self.disconnect_delay=0
        # ADR-0016 firmware: connecting does not claim; the first Setpoint/Control write does.
        self.claim_on_write=False;self.busy=False
        self.chars={u:SimpleNamespace(properties=p,max_write_without_response_size=244) for u,p in {
            SETPOINT:['write-without-response'],CONTROL:['write'],CAPS:['read'],
            STATUS:['read','notify'],ECHO:['write-without-response','notify']}.items()}
        self.services=SimpleNamespace(get_service=lambda u:object() if u==SERVICE else None,
                                      get_characteristic=lambda u:self.chars.get(u))
    async def connect(self):self.is_connected=True
    async def disconnect(self):
        self.disconnected=True;await asyncio.sleep(self.disconnect_delay);self.is_connected=False
    async def read_gatt_char(self,uuid):
        await asyncio.sleep(self.read_delay)
        if uuid==NETWORK:return bytes([2,0,0,0,192,168,1,50])
        return self.caps if uuid==CAPS else self.status
    async def start_notify(self,uuid,callback):self.callbacks[uuid]=callback
    async def write_gatt_char(self,uuid,data,response):
        if self.hang_writes:await asyncio.sleep(10)
        if self.fail_writes:raise BleakError('injected write failure')
        if response:await asyncio.sleep(self.write_delay)
        if self.busy and uuid==CONTROL:raise BleakError('ATT 0x13: another controller owns the car')
        if self.claim_on_write and self.status==b'\x00\x00\x00' and uuid in (SETPOINT,CONTROL):
            self.status=b'\x01\x00\x00'
        self.writes.append((uuid,bytes(data),response))
        if uuid==CONTROL:
            if data==b'\x01':
                if self.refuse_arm:raise BleakError('ARM refused')
                self.status=b'\x02\x00\x00'
                if self.arm_hook:self.arm_hook()
            else:self.status=b'\x03\x01\x00'
            if STATUS in self.callbacks:self.callbacks[STATUS](None,self.status)
        if uuid==ECHO and not self.drop_echo:self.callbacks[ECHO](None,data)


class BleTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.reader,self.writer=os.pipe();os.set_blocking(self.reader,False)
        self.peer=Peer();self.output=[];self.find=AsyncMock(return_value=self.peer)
        self.alert=Mock(return_value=True)
        self.client=BleClient(NAME,self.reader,self.output.append,find=self.find,alert=self.alert)
        self.task=None
    async def asyncTearDown(self):
        if self.task and not self.task.done():
            self.client.input(b'\x04')
            await asyncio.wait_for(self.task,2)
        os.close(self.reader);os.close(self.writer)
    async def until(self,predicate,timeout=2):
        end=time.monotonic()+timeout
        while not predicate():
            if self.task and self.task.done():await self.task
            self.assertLess(time.monotonic(),end,self.output[-8:])
            await asyncio.sleep(.01)
    async def start(self):
        self.task=asyncio.create_task(self.client.run())
        await self.until(lambda:self.client.ready)
    def key(self,keys):os.write(self.writer,keys)
    async def arm(self):
        self.key(b'r');await self.until(lambda:self.client.policy.armed)
    async def finish(self):
        self.key(b'\x04');await asyncio.wait_for(self.task,2)

    async def test_startup_six_directions_expiry_stop_and_quit(self):
        await self.start();self.key(b'w');await asyncio.sleep(.07)
        # The only control write before R is the claiming STOP: no ARM and no motion.
        self.assertFalse(any((u==CONTROL and d!=b'\x02') or (u==SETPOINT and any(vector(d))) for u,d,_ in self.peer.writes))
        await self.arm()
        for key,want in [(b'w',(.1,0,0)),(b's',(-.1,0,0)),(b'a',(0,.1,0)),
                         (b'd',(0,-.1,0)),(b'q',(0,0,.3)),(b'e',(0,0,-.3))]:
            start=len(self.peer.writes);self.key(key)
            await self.until(lambda:any(u==SETPOINT and vector(d)==want for u,d,_ in self.peer.writes[start:]))
        await asyncio.sleep(.3)
        self.assertEqual(vector([d for u,d,_ in self.peer.writes if u==SETPOINT][-1]),(0,0,0))
        self.key(b' w');await self.until(lambda:'STOP acknowledged' in self.output)
        start=len(self.peer.writes);self.key(b'w');await asyncio.sleep(.07)
        self.assertFalse(any(u==SETPOINT and any(vector(d)) for u,d,_ in self.peer.writes[start:]))
        await self.finish();self.assertTrue(self.client.cleanup_ok);self.assertTrue(self.peer.disconnected)
        self.assertTrue(all(len(d)==72 and not response for u,d,response in self.peer.writes if u==SETPOINT))

    async def test_space_during_arm_prevents_late_readback_enabling_motion(self):
        await self.start();self.peer.arm_hook=lambda:self.client.input(b' ')
        self.key(b'rw');await self.until(lambda:'STOP acknowledged' in self.output)
        self.assertFalse(self.client.policy.armed)
        self.assertFalse(any(u==SETPOINT and any(vector(d)) for u,d,_ in self.peer.writes))

    async def test_refused_arm_cannot_enable_motion(self):
        await self.start();self.peer.refuse_arm=True;self.key(b'rw')
        await self.until(lambda:any('control refused' in s for s in self.output))
        self.assertFalse(self.client.policy.armed)
        self.assertFalse(any(u==SETPOINT and any(vector(d)) for u,d,_ in self.peer.writes))

    async def test_valid_control_roundtrips_do_not_look_like_a_scheduler_stall(self):
        write=self.peer.write_gatt_char;read=self.peer.read_gatt_char
        async def delayed_write(*args,**kwargs):
            await asyncio.sleep(.11)
            return await write(*args,**kwargs)
        async def delayed_read(*args,**kwargs):
            await asyncio.sleep(.11)
            return await read(*args,**kwargs)
        self.peer.write_gatt_char=delayed_write;self.peer.read_gatt_char=delayed_read
        await self.start();await self.arm()
        self.assertFalse(self.task.done())

    async def test_disconnect_reconnect_ignores_old_callbacks_and_outage_keys(self):
        await self.start();await self.arm()
        old=self.peer;old_status=old.callbacks[STATUS];old_echo=old.callbacks[ECHO]
        replacement=Peer();self.find.return_value=replacement
        old.is_connected=False
        await self.until(lambda:self.alert.called)
        self.key(b'rw')
        await self.until(lambda:self.client.peer is replacement and self.client.ready,timeout=3)
        old_status(None,b'\x02\x00\x00');old_echo(None,struct.pack('<QI',1,1))
        self.key(b'w');await asyncio.sleep(.08)
        self.assertFalse(self.client.policy.armed)
        self.assertFalse(any((u==CONTROL and d!=b'\x02') or (u==SETPOINT and any(vector(d))) for u,d,_ in replacement.writes))
        self.assertEqual(self.alert.call_count,1)
        await self.arm();self.key(b'w')
        await self.until(lambda:any(u==SETPOINT and any(vector(d)) for u,d,_ in replacement.writes))

    async def test_name_discovery_pins_the_first_address_for_retries(self):
        await self.start();self.peer.is_connected=False
        await self.until(lambda:self.find.await_count>=2,timeout=3)
        self.assertEqual(self.find.await_args.args[0],self.peer.address)

    async def test_missing_echo_retries_and_stays_disarmed_until_fresh_echo(self):
        self.peer.drop_echo=True
        self.task=asyncio.create_task(self.client.run())
        await self.until(lambda:any('Echo reply timeout' in s for s in self.output))
        self.assertFalse(self.client.ready);self.assertEqual(self.alert.call_count,0)
        self.peer.drop_echo=False;self.key(b'rw')
        await self.until(lambda:self.client.ready,timeout=3)
        self.assertFalse(self.client.policy.armed)
        self.assertFalse(any(u==CONTROL and d==b'\x01' for u,d,_ in self.peer.writes))

    async def test_small_mtu_bad_caps_and_missing_service_fail_closed(self):
        for invalid in ('mtu','caps','service','characteristic','properties'):
            peer=Peer()
            if invalid=='mtu':peer.mtu_size=23;peer.chars[SETPOINT].max_write_without_response_size=20
            if invalid=='caps':peer.caps=b'bad'
            if invalid=='service':peer.services.get_service=lambda _:None
            if invalid=='characteristic':del peer.chars[ECHO]
            if invalid=='properties':peer.chars[CONTROL].properties=[]
            c=BleClient(NAME,self.reader,self.output.append,find=AsyncMock(return_value=peer))
            with self.subTest(invalid=invalid),self.assertRaises(ClientError):await c.run()
            self.assertTrue(peer.disconnected)
            self.assertFalse(any(u==CONTROL or (u==SETPOINT and any(vector(d))) for u,d,_ in peer.writes))

    async def test_bluez_write_capacity_can_establish_mtu_when_reported_view_is_23(self):
        self.peer.mtu_size=23;self.peer.chars[SETPOINT].max_write_without_response_size=72
        await self.start()
        self.assertTrue(any('mtu=75' in s for s in self.output))

    async def test_quit_during_discovery_cancels_and_returns(self):
        async def pending(_):await asyncio.sleep(10)
        self.client.find=pending;self.task=asyncio.create_task(self.client.run())
        await asyncio.sleep(.02);await self.finish()
        self.assertFalse(self.client.cleanup_ok)

    async def test_quit_during_connect_still_attempts_disconnect(self):
        async def pending():await asyncio.sleep(10)
        self.peer.connect=pending
        self.task=asyncio.create_task(self.client.run())
        await self.until(lambda:self.client.peer is self.peer)
        await self.finish()
        self.assertTrue(self.peer.disconnected)

    async def test_repeated_failures_alert_once_then_new_loss_alerts_again(self):
        await self.start()
        replacement=Peer();replacement.drop_echo=True;self.find.return_value=replacement
        self.peer.is_connected=False
        await self.until(lambda:self.find.await_count>=3,timeout=4)
        self.assertEqual(self.alert.call_count,1)
        replacement.drop_echo=False
        await self.until(lambda:self.client.ready,timeout=3)
        replacement.is_connected=False
        await self.until(lambda:self.alert.call_count==2)

    async def test_failed_or_stalled_cleanup_still_disconnects_within_budget(self):
        for hang in (False,True):
            self.peer=Peer();self.client.peer=self.peer;await self.peer.connect()
            from apps.teleop.policy import Policy
            from apps.teleop.protocol import capabilities
            self.client.policy=Policy(capabilities(CAPS_VALUE))
            self.peer.fail_writes=not hang;self.peer.hang_writes=hang
            self.peer.status=b'\x03\x01\x00' # An old stopped read alone is not acknowledgement.
            started=time.monotonic();self.assertFalse(await self.client.cleanup())
            self.assertLess(time.monotonic()-started,.5)
            self.assertTrue(self.peer.disconnected)
            self.assertIn('STOP unconfirmed',self.output[-1])

    async def test_claim_on_write_firmware_is_claimed_with_an_acknowledged_stop(self):
        self.peer.claim_on_write=True;self.peer.status=b'\x00\x00\x00'
        await self.start()
        self.assertEqual(self.peer.writes[0][:3],(CONTROL,b'\x02',True),'claims with an acknowledged STOP')
        self.assertFalse(any(u==CONTROL and d==b'\x01' for u,d,_ in self.peer.writes),'never arms by itself')
        await self.arm()
        self.key(b'w');await self.until(lambda:any(u==SETPOINT and vector(d)==(.1,0,0) for u,d,_ in self.peer.writes))

    async def test_mode_option_writes_the_mode_control_and_shows_the_network(self):
        self.peer.chars[NETWORK]=SimpleNamespace(properties=['read','notify'],max_write_without_response_size=0)
        self.client=BleClient(NAME,self.reader,self.output.append,find=self.find,alert=self.alert,mode='car')
        await self.start()
        controls=[d for u,d,_ in self.peer.writes if u==CONTROL]
        self.assertEqual(controls[:2],[b'\x02',b'\x04'],'claiming STOP, then Car network')
        self.assertIn('Car network: mode=home pending=none fallback=none ip=192.168.1.50',self.output)
        self.assertTrue(any('Mode car-network pending' in o for o in self.output))

    async def test_older_firmware_without_network_characteristic_still_connects(self):
        await self.start()
        self.assertFalse(any(o.startswith('Car network:') for o in self.output))

    async def test_busy_car_is_not_mistaken_for_a_claim(self):
        self.peer.claim_on_write=True;self.peer.status=b'\x02\x00\x00';self.peer.busy=True  # a Wi-Fi owner, armed
        self.task=asyncio.create_task(self.client.run())
        await self.until(lambda:any('another controller owns the car' in o for o in self.output))
        self.assertFalse(self.client.ready)
        self.assertFalse(any(u==SETPOINT and any(vector(d)) for u,d,_ in self.peer.writes))

    async def test_measured_rover_latencies_confirm_stop_within_budget(self):
        # 2026-10-22 rover timing: STOP write 75-124 ms, Status read 89-93 ms, BlueZ disconnect 2.1-2.6 s.
        self.peer=Peer();self.client.peer=self.peer;await self.peer.connect()
        from apps.teleop.policy import Policy
        from apps.teleop.protocol import capabilities
        self.client.policy=Policy(capabilities(CAPS_VALUE))
        self.peer.write_delay=.12;self.peer.read_delay=.095;self.peer.disconnect_delay=2.3
        started=time.monotonic();self.assertTrue(await self.client.cleanup())
        self.assertLess(time.monotonic()-started,.5)
        self.assertTrue(self.peer.disconnected)
        self.assertIn('STOP acknowledged',self.output[-1])
        self.assertIn('BLE disconnect requested (unconfirmed)',self.output[-1])

    async def test_malformed_or_unsolicited_status_never_arms(self):
        await self.start()
        for data in (b'',b'\x09\x00\x00',b'\x02\x01\x00',b'\x02\x00\x00'):
            self.peer.callbacks[STATUS](None,data)
        self.assertFalse(self.client.policy.armed)

    async def test_late_echo_cannot_cancel_the_expired_reply_deadline(self):
        await self.start()
        nonce=struct.pack('<QI',self.client.generation,999)
        self.client.echoes[nonce]=self.client.clock()-1
        self.client.notification(self.peer,self.client.generation,ECHO,nonce)
        with self.assertRaises(LinkError):self.client.check()


class DiscoveryTests(unittest.IsolatedAsyncioTestCase):
    async def test_ambiguous_name_fails_and_mac_selects_exactly_one_device(self):
        a=SimpleNamespace(address='AA:00:00:00:00:01',name=NAME)
        b=SimpleNamespace(address='AA:00:00:00:00:02',name=NAME)
        adv=SimpleNamespace(local_name=NAME)
        module=SimpleNamespace(BleakScanner=SimpleNamespace(discover=AsyncMock(return_value={'a':(a,adv),'b':(b,adv)})),BleakClient=Mock())
        with patch.dict('sys.modules',{'bleak':module}):
            with self.assertRaises(ClientError):await discover(NAME)
            await discover(a.address)
            self.assertIs(module.BleakClient.call_args.args[0],a)

    async def test_missing_bleak_has_actionable_error(self):
        with patch.dict('sys.modules',{'bleak':None}):
            with self.assertRaisesRegex(ClientError,'project nix-shell'):await discover(NAME)


class BleTerminalTests(unittest.TestCase):
    def setUp(self):
        code='''
from apps.teleop import ble
from apps.teleop.tests.test_ble import Peer
from apps.teleop.__main__ import main
original=ble.BleClient
async def find(_):return Peer()
def client(*args,**kwargs):return original(*args,find=find,alert=lambda:True,**kwargs)
ble.BleClient=client
raise SystemExit(main(['--ble']))
'''
        self.tty=Terminal(None,[sys.executable,'-u','-c',code])
        self.tty.expect('Connected disarmed')
    def tearDown(self):self.tty.close()
    def test_ble_cli_arm_stop_quit_restores_terminal(self):
        self.tty.key(b'r');self.tty.expect('local=armed')
        self.tty.key(b'w ');self.tty.expect('STOP acknowledged')
        self.assertEqual(self.tty.finish(),0)
        self.assertEqual(termios.tcgetattr(self.tty.slave),self.tty.before)
        self.assertIn(b'BLE disconnect confirmed',self.tty.output)
    def test_ble_termination_signal_cleans_up_and_restores_terminal(self):
        self.tty.proc.send_signal(signal.SIGTERM)
        self.tty.proc.wait(timeout=3);self.tty.finish(None)
        self.assertEqual(self.tty.proc.returncode,1)
        self.assertEqual(termios.tcgetattr(self.tty.slave),self.tty.before)
        self.assertIn(b'BLE disconnect confirmed',self.tty.output)
    def test_cli_rejects_conflicting_modes_and_invalid_ble_address(self):
        for args in (['--ble','--target','127.0.0.1:7777'],['--ble','not-a-mac']):
            result=subprocess.run([sys.executable,'-m','apps.teleop',*args],cwd=ROOT,capture_output=True,timeout=2)
            self.assertEqual(result.returncode,2)
