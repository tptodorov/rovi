import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Session } from '../session.js';
import { ZERO } from '../wire.js';
import { fakeBle, pending } from './fake-ble.js';

const flush = () => new Promise(resolve => setImmediate(resolve));
const motion = bytes => { const view = new DataView(Uint8Array.from(bytes).buffer); return [24, 32, 64].map(offset => view.getFloat64(offset, true)); };
const samples = fake => fake.calls.filter(x => x[0] === 'setpoint').map(x => motion(x[1]));
async function ready() {
  const fake = fakeBle(); let time = 0, active = true;
  const session = new Session({ bluetooth: fake.bluetooth, protocol: 'https:', now: () => time, active: () => active });
  await session.choose(); await flush(); assert.equal(session.phase, 'ready');
  return { fake, session, setTime: value => { time = value; }, inactive: () => { active = false; },
    async advance(delta) { time += delta; session.tick(); await flush(); } };
}

test('connect sends only zero; explicit zero/ARM/read then a new press permits gentle motion', async () => {
  const { session, fake, advance } = await ready();
  assert.equal(session.localArm, false); assert.deepEqual(samples(fake), [[0, 0, 0]]);
  session.press(1, 'forward'); session.arm(); await flush();
  assert.equal(session.localArm, true); assert.deepEqual(session.requested, ZERO);
  await advance(50); assert.deepEqual(samples(fake).at(-1), [0, 0, 0]);
  session.release(1); session.press(2, 'forward'); await advance(50);
  assert.deepEqual(samples(fake).at(-1), [0.1, 0, 0]);
  session.release(2); await advance(50); assert.deepEqual(samples(fake).at(-1), [0, 0, 0]);
  session.seq = 0xffffffff; await advance(50); await advance(50);
  const tail = fake.calls.filter(x => x[0] === 'setpoint').slice(-2);
  assert.equal(new DataView(Uint8Array.from(tail[0][1]).buffer).getUint32(0, true), 0xffffffff);
  assert.equal(new DataView(Uint8Array.from(tail[1][1]).buffer).getUint32(0, true), 0);
});

test('Armed notification cannot substitute for the ARM write and its subsequent read', async () => {
  const { fake, session } = await ready(), write = pending(), read = pending();
  fake.chars.control.writeValueWithResponse = () => write.promise;
  fake.chars.status.readValue = () => read.promise;
  session.arm(); await flush(); fake.notify([2, 0, 0]);
  assert.equal(session.localArm, false);
  write.resolve(); await flush(); assert.equal(session.localArm, false);
  read.resolve(new DataView(Uint8Array.of(2, 0, 0).buffer)); await flush();
  assert.equal(session.localArm, true);
});

test('STOP supersedes a pending ARM; a late ARM cannot grant motion', async () => {
  const { fake, session } = await ready(), gate = pending();
  const original = fake.chars.control.writeValueWithResponse;
  fake.chars.control.writeValueWithResponse = bytes => bytes[0] === 1 ? gate.promise.then(() => original(bytes)) : original(bytes);
  session.arm(); await flush(); session.press(1, 'forward'); session.stop();
  gate.resolve(); await flush();
  assert.equal(session.localArm, false); assert.equal(session.reported.reason, 'Stop');
  assert.deepEqual(session.requested, ZERO); assert.equal(session.phase, 'ready');
  assert.ok(samples(fake).every(values => values.every(x => x === 0)));
});

test('a safety stop latches off, and a fresh ARM cannot replay the old finger', async () => {
  const { fake, session, advance } = await ready(); session.arm(); await flush();
  session.press(1, 'left'); await advance(50); fake.notify([3, 2, 0]);
  assert.equal(session.localArm, false); await advance(50);
  assert.deepEqual(samples(fake).at(-1), [0, 0, 0]);
  session.arm(); await flush(); assert.equal(session.localArm, true);
  assert.deepEqual(session.requested, ZERO);
});

test('UI renewal expiry and a 200 ms scheduler gap cannot replay motion', async () => {
  const { fake, session, advance } = await ready(); session.arm(); await flush(); session.press(1, 'forward');
  for (let n = 0; n < 5; n++) await advance(50);
  assert.deepEqual(samples(fake).at(-1), [0, 0, 0]);
  const before = samples(fake).length;
  await advance(200); assert.equal(session.phase, 'disconnected');
  assert.ok(samples(fake).slice(before).every(v => v.every(x => x === 0)));
});

test('a pending write completing at 200 ms closes before another nonzero submission, even before tick', async () => {
  const { fake, session, advance, setTime } = await ready(); session.arm(); await flush(); session.press(1, 'forward');
  const gate = pending(), original = fake.chars.setpoint.writeValueWithoutResponse;
  fake.chars.setpoint.writeValueWithoutResponse = bytes => { void original(bytes); return gate.promise; };
  await advance(50); const before = samples(fake).length;
  setTime(250); session.hold.renew(249); gate.resolve(); await flush();
  assert.equal(session.phase, 'disconnected');
  assert.ok(samples(fake).slice(before).every(v => v.every(x => x === 0)));
});

test('cleanup attempts STOP independently when zero never resolves, and disconnects by 500 ms', async () => {
  const { fake, session, setTime } = await ready(), zero = pending();
  fake.chars.setpoint.writeValueWithoutResponse = () => zero.promise;
  const closing = session.close('Test shutdown'); await flush();
  assert.ok(fake.calls.some(([kind, code]) => kind === 'control' && code === 2));
  setTime(500); session.tick(); await closing;
  assert.equal(fake.server.connected, false); assert.equal(session.phase, 'disconnected');
  zero.resolve(); await flush(); assert.equal(session.localArm, false);
});

test('zero rejection still attempts STOP; cleanup Status is device-reported, not physical proof', async () => {
  const { fake, session } = await ready();
  fake.chars.setpoint.writeValueWithoutResponse = async () => { throw new Error('write failed'); };
  await session.close('Test failure');
  assert.ok(fake.calls.some(([kind, code]) => kind === 'control' && code === 2));
  assert.match(session.message, /physical stop\/release not verified/);
});

test('malformed/Open/unexpected Armed and read errors end the session without auto-reconnect', async () => {
  for (const bytes of [[255], [0, 0, 0], [2, 0, 0]]) {
    const { fake, session } = await ready(); fake.notify(bytes); await flush();
    assert.equal(session.phase, 'disconnected'); assert.equal(session.localArm, false);
    assert.equal(fake.calls.filter(x => x[0] === 'picker').length, 1);
  }
  const { fake, session, advance } = await ready();
  fake.chars.status.readValue = async () => { throw new Error('read refused'); };
  for (let n = 0; n < 11; n++) await advance(50);
  assert.equal(session.phase, 'disconnected');
});

test('late picker selection or connect cannot start a cancelled generation', async () => {
  const fake = fakeBle(), picker = pending();
  fake.bluetooth.requestDevice = () => picker.promise;
  const session = new Session({ bluetooth: fake.bluetooth, protocol: 'https:' });
  const choosing = session.choose(); await session.close('Cancel picker');
  picker.resolve(fake.device); await choosing; await flush();
  assert.equal(fake.calls.some(x => x[0] === 'connect'), false);
});

test('setup deadline is checked again at completion, before zero or ready state', async () => {
  const fake = fakeBle(), connect = pending(); let time = 0;
  fake.server.connect = () => connect.promise;
  const session = new Session({ bluetooth: fake.bluetooth, protocol: 'https:', now: () => time });
  await session.choose(); await flush(); time = 10000;
  fake.server.connected = true; connect.resolve(fake.server); await flush();
  assert.equal(session.phase, 'disconnected');
  assert.match(session.message, /timed out/);
});

test('link loss while armed and held ends the session; no motion or reconnect follows', async () => {
  const { fake, session, advance } = await ready(); session.arm(); await flush();
  session.press(1, 'forward'); await advance(50); assert.deepEqual(samples(fake).at(-1), [0.1, 0, 0]);
  const sent = samples(fake).length;
  fake.server.connected = false; fake.device.dispatchEvent(new Event('gattserverdisconnected')); await flush();
  await advance(50);
  assert.equal(session.phase, 'disconnected'); assert.equal(session.localArm, false);
  assert.match(session.message, /connection lost/); assert.equal(samples(fake).length, sent);
  assert.equal(fake.calls.filter(x => x[0] === 'picker').length, 1);
});

test('a hung ARM write or Status read ends the session within one second', async () => {
  for (const hang of ['control', 'read']) {
    const { fake, session, advance } = await ready(), never = pending();
    if (hang === 'control') { fake.chars.control.writeValueWithResponse = () => never.promise; session.arm(); }
    else fake.chars.status.readValue = () => never.promise;
    for (let t = 0; t < 1000 && session.phase === 'ready'; t += 25) await advance(25);
    assert.notEqual(session.phase, 'ready', hang); assert.equal(session.localArm, false, hang);
  }
});

test('stream stats record writes, largest submission gap and slowest write for P2 evidence', async () => {
  const { fake, session, setTime } = await ready(); let t = 0;
  const original = fake.chars.setpoint.writeValueWithoutResponse;
  fake.chars.setpoint.writeValueWithoutResponse = async bytes => { setTime(t += 7); return original(bytes); };
  for (let n = 0; n < 4; n++) { setTime(t += 60); session.tick(); await flush(); }
  assert.equal(session.stats.writes, samples(fake).length);
  assert.ok(session.stats.maxGap >= 60 && session.stats.maxGap < 200, String(session.stats.maxGap));
  assert.equal(session.stats.maxWrite, 7);
});
