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
  const before = fake.calls.length;  // after the claiming STOP
  fake.chars.setpoint.writeValueWithoutResponse = () => zero.promise;
  const closing = session.close('Test shutdown'); await flush();
  assert.ok(fake.calls.slice(before).some(([kind, code]) => kind === 'control' && code === 2));
  setTime(500); session.tick(); await closing;
  assert.equal(fake.server.connected, false); assert.equal(session.phase, 'disconnected');
  zero.resolve(); await flush(); assert.equal(session.localArm, false);
});

test('zero rejection still attempts STOP; cleanup Status is device-reported, not physical proof', async () => {
  const { fake, session } = await ready();
  const before = fake.calls.length;  // after the claiming STOP
  fake.chars.setpoint.writeValueWithoutResponse = async () => { throw new Error('write failed'); };
  await session.close('Test failure');
  assert.ok(fake.calls.slice(before).some(([kind, code]) => kind === 'control' && code === 2));
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

const physicalNeutral = { alpha: 350, beta: 0, gamma: 0 };
const physicalTilt = { alpha: 10, beta: -20, gamma: -20 };
async function physicalReady() {
  const state = await ready(); state.session.selectMode('physical'); await flush();
  state.session.physical.enable(); state.session.sensor(physicalNeutral);
  return state;
}
test('Physical needs neutral/ARM and explicit START; all three axes use the same wire stream', async () => {
  const { session, fake, advance } = await physicalReady();
  assert.equal(session.localArm, false); session.togglePhysical(); session.arm(); await flush();
  session.sensor(physicalTilt); await advance(50); assert.deepEqual(samples(fake).at(-1), [0, 0, 0]);
  session.togglePhysical(); await advance(50); assert.deepEqual(samples(fake).at(-1), [0, 0, 0]);
  session.sensor(physicalNeutral); session.togglePhysical(); session.sensor(physicalTilt); await advance(50);
  assert.deepEqual(samples(fake).at(-1), [.1, .1, .3]);
  session.togglePhysical(); await advance(50); assert.deepEqual(samples(fake).at(-1), [0, 0, 0]);
  session.selectMode('buttons'); await flush(); assert.equal(session.localArm, false);
  assert.equal(session.physical.enabled, false); assert.equal(session.reported.reason, 'Stop');
  assert.deepEqual(session.requested, ZERO);
});
test('mode change supersedes pending Physical ARM and late results cannot arm Buttons', async () => {
  const { fake, session } = await physicalReady(), gate = pending();
  const original = fake.chars.control.writeValueWithResponse;
  fake.chars.control.writeValueWithResponse = bytes => bytes[0] === 1 ? gate.promise.then(() => original(bytes)) : original(bytes);
  session.arm(); await flush(); session.togglePhysical(); session.selectMode('buttons');
  gate.resolve(); await flush();
  assert.equal(session.mode, 'buttons'); assert.equal(session.localArm, false);
  assert.equal(session.reported.reason, 'Stop'); assert.equal(session.physical.neutral, null);
  assert.ok(samples(fake).every(values => values.every(value => value === 0)));
});
test('START during pending ARM is discarded rather than replayed after confirmation', async () => {
  const { fake, session } = await physicalReady(), gate = pending();
  const original = fake.chars.control.writeValueWithResponse;
  fake.chars.control.writeValueWithResponse = bytes => bytes[0] === 1 ? gate.promise.then(() => original(bytes)) : original(bytes);
  session.arm(); await flush(); session.togglePhysical();
  gate.resolve(); await flush(); session.sensor(physicalTilt);
  assert.equal(session.localArm, true); assert.equal(session.physical.driving, false);
  assert.deepEqual(session.requested, ZERO);
});
test('sensor expiry closes while UI and stream continue, and returning samples cannot recover', async () => {
  const { session, advance } = await physicalReady(); session.arm(); await flush(); session.togglePhysical();
  for (let at = 50; at <= 300; at += 50) { session.physical.renew(at); await advance(50); }
  assert.equal(session.phase, 'disconnected'); assert.equal(session.localArm, false);
  session.sensor(physicalTilt); assert.equal(session.physical.enabled, false);
  assert.deepEqual(session.requested, ZERO);
});
test('invalid sensor, posture and foreground expiry independently close an armed Physical session', async () => {
  for (const failure of ['null', 'posture', 'frame']) {
    const { session, advance, setTime } = await physicalReady(); session.arm(); await flush(); session.togglePhysical();
    if (failure === 'frame') {
      for (const at of [100, 200, 250]) { setTime(at); session.sensor(physicalTilt); }
    } else session.sensor({ ...physicalTilt, beta: failure === 'null' ? null : 61 });
    await advance(0);
    assert.equal(session.phase, 'disconnected', failure); assert.equal(session.localArm, false, failure);
    assert.equal(session.physical.neutral, null, failure);
  }
});
test('Physical ARM without sensors is refused locally without an ARM write', async () => {
  const { session, fake } = await ready(); session.selectMode('physical'); await flush();
  session.arm(); await flush(); assert.equal(session.localArm, false);
  assert.equal(fake.calls.some(([kind, value]) => kind === 'control' && value === 1), false);
});

test('one-tap Physical ARM rejects stale, invalid or unsafe neutral and each fresh ARM recenters', async () => {
  for (const bad of ['stale', 'invalid', 'posture']) {
    const { session, fake, setTime } = await ready(); session.selectMode('physical'); await flush();
    session.physical.enable(); session.sensor({ ...physicalNeutral, beta: bad === 'posture' ? 31 : bad === 'invalid' ? null : 0 });
    if (bad === 'stale') setTime(250);
    session.arm(); await flush(); assert.equal(session.localArm, false, bad);
    assert.equal(fake.calls.some(([kind, value]) => kind === 'control' && value === 1), false, bad);
  }
  const { session } = await physicalReady(); session.arm(); await flush();
  session.stop(); await flush(); session.sensor(physicalTilt); session.arm(); await flush();
  assert.equal(session.localArm, true); session.togglePhysical();
  assert.deepEqual(session.requested, ZERO);
});

test('invalid values or a reference change still close Physical while released', async () => {
  for (const sample of [{ ...physicalNeutral, alpha: null }, { ...physicalNeutral, absolute: true }]) {
    const { session } = await physicalReady(); session.arm(); await flush();
    session.sensor(sample); await flush();
    assert.equal(session.localArm, false); assert.equal(session.phase, 'disconnected');
    assert.deepEqual(session.requested, ZERO);
  }
});

test('Physical ARM captures neutral in one explicit action and never replays an early START', async () => {
  const { session, fake, advance } = await ready(); session.selectMode('physical'); await flush();
  session.physical.enable(); session.sensor(physicalTilt); session.togglePhysical();
  session.arm(); await flush();
  assert.equal(session.localArm, true);
  assert.deepEqual(session.physical.neutral, { ...physicalTilt, reference: null, at: 0 });
  assert.deepEqual(session.requested, ZERO);
  session.sensor({ ...physicalTilt, beta: -30 }); await advance(50);
  assert.deepEqual(samples(fake).at(-1), [0, 0, 0]);
  session.sensor(physicalTilt); session.togglePhysical();
  session.sensor({ ...physicalTilt, beta: -30 }); await advance(50);
  assert.ok(samples(fake).at(-1)[0] > 0);
});

test('paused Physical input tolerates idle sensor gaps and reading posture without disarming', async () => {
  const { session, advance } = await physicalReady(); session.arm(); await flush();
  const neutral = session.physical.neutral;
  // Scrolling may interrupt sensors or tilt the phone while the tilt control is paused.
  for (let at = 50; at <= 350; at += 50) await advance(50);
  assert.equal(session.localArm, true); assert.equal(session.phase, 'ready');
  assert.deepEqual(session.requested, ZERO); assert.equal(session.physical.neutral, neutral);
  session.togglePhysical(); session.sensor({ ...physicalNeutral, beta: 75 }); await advance(50);
  assert.equal(session.localArm, true); assert.deepEqual(session.requested, ZERO);
  session.sensor(physicalTilt); await advance(50);
  assert.deepEqual(session.requested, ZERO); // The rejected START cannot become driving later.
  session.togglePhysical(); assert.deepEqual(session.requested, ZERO);
  session.sensor(physicalNeutral); session.togglePhysical(); session.sensor(physicalTilt); await advance(50);
  assert.ok(session.requested.vx > 0);
});
