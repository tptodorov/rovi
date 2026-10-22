"""Native BLE keyboard loop; connecting claims ownership, disconnecting releases."""
import asyncio
import struct
import warnings
from .audio import beep
from .policy import Policy, ZERO
from .protocol import ARM, STOP, ClientError, LinkError, capabilities
from .runtime import milliseconds, read_keys
from apps.client.wire import Status, ble_setpoint

try:
    from bleak.exc import BleakError
except ImportError:  # UDP and fake-GATT tests do not require Bleak.
    class BleakError(Exception):
        pass

SERVICE = 'd47a0010-45b2-4d19-8db0-6bd87e9c0001'
SETPOINT, CONTROL, CAPS, STATUS, ECHO = [
    f'd47a00{i:02x}-45b2-4d19-8db0-6bd87e9c0001' for i in range(0x11,0x16)
]
NAME = 'Rovi M8'


async def discover(address):
    try:
        from bleak import BleakClient, BleakScanner
    except ImportError:
        raise ClientError('BLE needs Bleak; run this command in the project nix-shell') from None
    seen = await BleakScanner.discover(timeout=4,return_adv=True)
    matches = [dev for dev,adv in seen.values()
               if (dev.address.upper()==address.upper() if address != NAME
                   else adv.local_name==NAME or dev.name==NAME)]
    if not matches:
        raise LinkError('BLE target not advertising; it may be owned by another controller')
    if len(matches) != 1:
        raise ClientError('multiple Rovi M8 devices found; use --ble MAC')
    return BleakClient(matches[0],timeout=5)


class BleClient:
    def __init__(self, address, input_fd, emit, *, find=discover, clock=milliseconds, alert=beep):
        self.address,self.input_fd,self.emit = address,input_fd,emit
        self.find,self.clock,self.alert = find,clock,alert
        self.peer = self.policy = self.task = None
        self.ready = self.quit = self.busy = self.connected_once = self.alerted = False
        self.generation = self.epoch = self.seq = self.nonce_seq = 0
        self.action = self.failure = self.status = self.status_at = self.echo_at = None
        self.echoes = {}
        self.cleanup_ok = False

    def input(self, keys):
        if any(k in keys for k in (3,4,27)):
            self.quit = True
            if self.policy:
                self.policy.clear()
            if self.task:
                self.task.cancel()
            return
        if not self.ready or not self.policy:
            return
        action = self.policy.input(keys,self.clock())
        if action in ('arm','stop'):
            self.epoch += 1
            if action == 'stop' or (not self.busy and self.action is None):
                self.action = (ARM if action=='arm' else STOP,self.epoch)

    def input_ready(self, discard=False):
        try:
            keys = read_keys(self.input_fd)
            if discard:
                keys = bytes(k for k in keys if k in (3,4,27))
            self.input(keys)
        except (ClientError,OSError) as exc:
            self.failure = exc
            self.input(b'\x04')

    def observe(self, data, control=None):
        try:
            status = Status.decode(data)
        except ValueError:
            return False
        if ((status.state=='Stopped') != (status.reason is not None)
                or status.error not in (None,'NotReady','Invalid')):
            return False
        self.status,self.status_at = status,self.clock()
        if status.state!='Armed' or status.reason or status.error:
            self.epoch += 1
        if self.policy:
            self.policy.observe(status,control)
        return True

    def notification(self, peer, generation, kind, data):
        if peer is not self.peer or generation != self.generation or not peer.is_connected:
            return
        if kind == STATUS:
            self.observe(data)
        elif kind == ECHO:
            deadline = self.echoes.get(bytes(data))
            if deadline is not None and self.clock() < deadline:
                del self.echoes[bytes(data)]
                self.echo_at = self.clock()
                if not self.ready:
                    self.input_ready(discard=True)
                    if self.quit:
                        return
                    self.ready = True
                    self.alerted = False
                    label = 'Reconnected' if self.connected_once else 'Connected'
                    self.connected_once = True
                    self.emit(f'{label} disarmed BLE generation={generation}; press R to arm')

    async def io(self, operation, timeout=.18):
        try:
            return await asyncio.wait_for(operation,timeout)
        except (OSError,asyncio.TimeoutError,BleakError) as exc:
            raise LinkError(f'BLE operation failed: {exc}') from exc

    def check(self):
        if not self.peer.is_connected:
            raise LinkError('BLE disconnected')
        now = self.clock()
        self.policy.iteration(now)
        if any(now>=deadline for deadline in self.echoes.values()):
            raise LinkError('BLE Echo reply timeout')

    async def setpoint(self, zero=False):
        self.check()
        self.seq = (self.seq+1) & 0xffffffff
        vector = ZERO if zero or not self.ready else self.policy.vector(self.clock())
        await self.io(self.peer.write_gatt_char(SETPOINT,ble_setpoint(self.seq,*vector),response=False))

    async def control(self, kind, epoch):
        self.busy = True
        try:
            await self.setpoint(zero=True)
            self.check()
            if kind==ARM and epoch != self.epoch:
                return
            try:
                await asyncio.wait_for(self.peer.write_gatt_char(CONTROL,bytes([1 if kind==ARM else 2]),response=True),.18)
            except BleakError as exc:
                if not self.peer.is_connected:
                    raise LinkError(f'BLE control disconnected: {exc}') from exc
                self.emit(f'BLE control refused: {exc}')
                self.observe(await self.io(self.peer.read_gatt_char(STATUS)))
                return
            except (OSError,asyncio.TimeoutError) as exc:
                raise LinkError(f'BLE control failed: {exc}') from exc
            self.check()
            data = await self.io(self.peer.read_gatt_char(STATUS))
            self.check()
            matched = kind if epoch==self.epoch else None
            if not self.observe(data,matched):
                raise ClientError('invalid BLE control Status')
            if kind==STOP:
                self.emit('STOP acknowledged' if self.status==Status('Stopped','Stop',None) else 'STOP unconfirmed/refused')
        finally:
            self.busy = False

    async def attempt(self):
        self.generation += 1
        self.peer = await self.io(self.find(self.address),5)
        if self.address==NAME:
            self.address = self.peer.address
            self.emit(f'BLE target pinned to {self.address}')
        await self.io(self.peer.connect(),5)
        required = {SETPOINT:{'write-without-response'},CONTROL:{'write'},CAPS:{'read'},
                    STATUS:{'read','notify'},ECHO:{'write-without-response','notify'}}
        if self.peer.services.get_service(SERVICE) is None:
            raise ClientError('Rovi BLE service missing')
        for uuid,props in required.items():
            char = self.peer.services.get_characteristic(uuid)
            if char is None or not props.issubset(char.properties):
                raise ClientError(f'incompatible BLE characteristic {uuid}')
        acquire = getattr(getattr(self.peer,'_backend',None),'_acquire_mtu',None)
        if acquire:
            try:
                await self.io(acquire(),1)
            except LinkError:
                pass  # The characteristic write capacity is the fallback view.
        char = self.peer.services.get_characteristic(SETPOINT)
        with warnings.catch_warnings():
            warnings.simplefilter('ignore')
            mtu = max(self.peer.mtu_size,char.max_write_without_response_size+3)
        if mtu < 75:
            raise ClientError(f'BLE ATT MTU {mtu} cannot carry a 72-byte setpoint')
        try:
            caps = capabilities(await self.io(self.peer.read_gatt_char(CAPS),1))
        except ValueError as exc:
            raise ClientError(f'invalid BLE capabilities: {exc}') from exc
        self.policy = Policy(caps)
        self.seq = 0
        if not self.observe(await self.io(self.peer.read_gatt_char(STATUS),1)) or self.status.state not in ('Claimed','Stopped'):
            raise ClientError('unexpected BLE claim Status')
        peer,generation = self.peer,self.generation
        for uuid in (STATUS,ECHO):
            await self.io(peer.start_notify(uuid,lambda _,data,k=uuid:self.notification(peer,generation,k,bytes(data))),1)
        self.emit(f'BLE claimed disarmed generation={generation} mtu={mtu} capabilities={dict(caps._asdict())}')
        stream_at = echo_at = display_at = 0
        while True:
            self.check()
            if self.action:
                action,self.action = self.action,None
                await self.control(*action)
            now = self.clock()
            if now>=stream_at:
                await self.setpoint()
                stream_at = self.clock()+50
            if now>=echo_at:
                self.check()
                self.nonce_seq = (self.nonce_seq+1) & 0xffffffff
                nonce = struct.pack('<QI',generation,self.nonce_seq)
                self.echoes[nonce] = self.clock()+500
                await self.io(peer.write_gatt_char(ECHO,nonce,response=False))
                self.check()
                echo_at = self.clock()+250
            if now>=display_at:
                age = lambda at:'none' if at is None else str(max(0,self.clock()-at))
                self.emit(f'connection={"connected" if self.ready else "verifying"} transport=BLE '
                          f'local={"armed" if self.policy.armed else "disarmed"} generation={generation} '
                          f'last_reported={self.status} status_age_ms={age(self.status_at)} '
                          f'echo_age_ms={age(self.echo_at)} requested_vx/vy/wz={self.policy.vector(self.clock())}')
                display_at = self.clock()+250
            await asyncio.sleep(.01)

    async def cleanup(self):
        self.ready = False
        self.generation += 1  # Invalidate notifications before disconnect and the next attempt.
        if self.policy:
            self.policy.clear()
        peer = self.peer
        confirmed = False
        writes_ok = True
        stop_written = False
        if peer is not None:
            # 500 ms budget minus scheduling margin. Rover GATT round trips take ~80-125 ms;
            # BlueZ disconnect takes ~2 s and completes after the client stops waiting.
            end = asyncio.get_running_loop().time()+.45
            def limit(op):
                return max(0,min(op,end-asyncio.get_running_loop().time()))
            if peer.is_connected and self.policy:
                self.seq = (self.seq+1) & 0xffffffff
                for uuid,data in ((SETPOINT,ble_setpoint(self.seq)),(CONTROL,b'\x02')):
                    try:
                        await self.io(peer.write_gatt_char(uuid,data,response=uuid==CONTROL),limit(.05 if uuid==SETPOINT else .17))
                        if uuid==CONTROL:
                            stop_written = True
                    except LinkError:
                        writes_ok = False
                try:
                    status = Status.decode(await self.io(peer.read_gatt_char(STATUS),limit(.15)))
                    confirmed = stop_written and status==Status('Stopped','Stop',None)
                except (LinkError,ValueError):
                    pass
            try:
                await self.io(peer.disconnect(),limit(.45))
            except LinkError:
                pass  # The request is sent; BlueZ completes it after the budget.
            disconnected = not peer.is_connected
            self.emit(f'Cleanup: STOP {"acknowledged" if confirmed else "unconfirmed"}; '
                      f'BLE disconnect {"confirmed" if disconnected else "requested (unconfirmed)"}; physical stop unverified')
            return confirmed and writes_ok
        return False

    async def run(self):
        loop = asyncio.get_running_loop()
        self.task = asyncio.current_task()
        loop.add_reader(self.input_fd,self.input_ready)
        self.emit('R arm | WASD translate | QE rotate | Space STOP | Esc/Ctrl-C quit; BLE starts disarmed')
        try:
            while not self.quit:
                try:
                    await self.attempt()
                except (LinkError,BleakError) as exc:
                    self.ready = False
                    if self.policy:
                        self.policy.clear()
                    self.emit(f'BLE connection unavailable: {exc}; motion cleared; reconnecting disarmed')
                    await self.cleanup()
                    self.peer = self.policy = self.action = None
                    self.echoes.clear()
                    self.status = self.status_at = self.echo_at = None
                    if self.connected_once and not self.alerted:
                        self.alerted = True
                        if not await asyncio.to_thread(self.alert):
                            self.emit('\aAudio alert unavailable; terminal bell attempted')
                    await asyncio.sleep(1)
        except asyncio.CancelledError:
            if not self.quit:
                raise
        finally:
            loop.remove_reader(self.input_fd)
            self.cleanup_ok = await self.cleanup()
        if self.failure:
            raise self.failure
