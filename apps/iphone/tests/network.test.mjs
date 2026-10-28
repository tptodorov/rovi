import { test } from 'node:test';
import assert from 'node:assert/strict';
import * as wire from '../wire.js';
import { Ble } from '../transport.js';
import { Session } from '../session.js';
import { fakeBle, pending } from './fake-ble.js';

const flush = () => new Promise(resolve => setImmediate(resolve));
async function ready(options) {
  const fake = fakeBle(options); let time = 0;
  const session = new Session({ bluetooth: fake.bluetooth, protocol: 'https:', now: () => time });
  await session.choose(); await flush(); assert.equal(session.phase, 'ready');
  return { fake, session, async advance(ms) { time += ms; session.tick(); await flush(); } };
}
const modeWrites = fake => fake.calls.filter(([kind, code]) => kind === 'control' && [3, 4].includes(code));
const noMotion = fake => {
  assert.equal(fake.calls.some(([kind, code]) => kind === 'control' && code === 1), false);
  for (const [kind, data] of fake.calls) if (kind === 'setpoint') {
    const bytes = new DataView(Uint8Array.from(data).buffer);
    assert.deepEqual([24, 32, 64].map(n => bytes.getFloat64(n, true)), [0, 0, 0]);
  }
};

test('Network decoder validates the eight-byte contract and preserves IPv4 octets', () => {
  assert.deepEqual(wire.network(Uint8Array.of(2, 1, 0, 0, 192, 168, 1, 193)),
    { mode: 'Home', pending: 'Home', reason: 'none', ip: '192.168.1.193' });
  assert.equal(wire.network(Uint8Array.of(0, 0, 6, 0, 0, 0, 0, 0)).reason, 'LinkLost');
  for (const bytes of [[3, 0, 0, 0, 0, 0, 0, 0], [2, 3, 0, 0, 0, 0, 0, 0],
    [2, 0, 8, 0, 0, 0, 0, 0], [2, 0, 0, 1, 0, 0, 0, 0], [2, 0]]) {
    assert.throws(() => wire.network(Uint8Array.from(bytes)));
  }
});

test('optional Network reads and notifications are copied and ignored after disconnect', async () => {
  const fake = fakeBle(), seen = [];
  const ble = new Ble(fake.device, () => {}, () => {}, bytes => seen.push(wire.network(bytes)));
  const initial = await ble.open();
  assert.equal(wire.network(initial.network).mode, 'Home');
  fake.notifyNetwork([0, 2, 5, 0, 0, 0, 0, 0]);
  assert.equal(seen.at(-1).pending, 'CarNetwork');
  assert.equal(wire.network(await ble.readNetwork()).reason, 'Router');
  ble.disconnect(); fake.notifyNetwork([2, 0, 0, 0, 1, 2, 3, 4]);
  assert.equal(seen.at(-1).mode, 'Fallback');
});

test('old firmware remains usable with network actions unavailable', async () => {
  const { fake, session } = await ready({ network: false });
  assert.equal(session.networkInfo, null);
  session.setNetwork('Home'); await flush();
  assert.equal(session.phase, 'ready'); assert.deepEqual(modeWrites(fake), []);
});

test('unknown network targets send no command', async () => {
  const { fake, session } = await ready();
  session.setNetwork('invalid'); await flush();
  assert.deepEqual(modeWrites(fake), []); assert.equal(session.phase, 'ready'); noMotion(fake);
});

for (const [target, code] of [['Home', 3], ['CarNetwork', 4]]) {
  test(`${target} is freshly confirmed before disconnect, with no ARM or motion`, async () => {
    const { fake, session } = await ready(); const before = fake.calls.length;
    session.setNetwork(target); await flush();
    assert.equal(session.phase, 'disconnected');
    const calls = fake.calls.slice(before), mode = calls.findIndex(c => c[0] === 'control' && c[1] === code);
    const confirmed = calls.findIndex((c, i) => i > mode && c[0] === 'network-read');
    assert.ok(mode >= 0 && confirmed > mode && calls.findIndex(c => c[0] === 'disconnect') > confirmed, calls);
    assert.equal(session.networkInfo.pending, target);
    assert.match(session.networkMessage, /Reconnect/); noMotion(fake);
  });
}

test('network changes are ignored while armed or another control is in flight', async () => {
  const { fake, session } = await ready();
  session.arm(); session.setNetwork('Home'); await flush();
  assert.equal(session.localArm, true); session.setNetwork('CarNetwork'); await flush();
  assert.deepEqual(modeWrites(fake), []);
});

test('network confirmation renews zero between GATT round trips without relaxing the stream guard', async () => {
  const fake = fakeBle(); let time = 0;
  const timers = [];
  const session = new Session({ bluetooth: fake.bluetooth, protocol: 'https:', now: () => time });
  await session.choose(); await flush();
  for (const [char, method] of [[fake.chars.control, 'writeValueWithResponse'], [fake.chars.status, 'readValue'], [fake.chars.network, 'readValue']]) {
    const original = char[method];
    char[method] = (...args) => new Promise(resolve => timers.push({ at: time + 125, resolve })).then(() => original(...args));
  }
  session.setNetwork('Home'); await flush();
  for (let n = 0; n < 40 && session.phase !== 'disconnected'; n++) {
    time += 25;
    for (const timer of timers.splice(0)) {
      if (timer.at <= time) timer.resolve(); else timers.push(timer);
    }
    session.tick(); await flush();
  }
  assert.equal(session.phase, 'disconnected');
  assert.match(session.networkMessage, /Home Wi-Fi requested/); noMotion(fake);
});

test('refused Home remains connected/disarmed with an explanation', async () => {
  const { fake, session } = await ready({ homeConfigured: false });
  session.setNetwork('Home'); await flush();
  assert.equal(session.phase, 'ready'); assert.equal(session.localArm, false);
  assert.match(session.networkMessage, /unavailable/i); noMotion(fake);
});

test('unconfirmed pending mode closes without claiming success', async () => {
  const { fake, session } = await ready();
  fake.chars.network.readValue = async () => new DataView(Uint8Array.of(2, 0, 0, 0, 0, 0, 0, 0).buffer);
  session.setNetwork('Home'); await flush();
  assert.equal(session.phase, 'disconnected');
  assert.match(session.networkMessage, /not confirmed/i); noMotion(fake);
});

test('a failed write cannot confirm an already-pending request even with error-free Status', async () => {
  const { fake, session } = await ready();
  fake.notifyNetwork([2, 1, 0, 0, 192, 168, 1, 193]);
  const write = fake.chars.control.writeValueWithResponse;
  fake.chars.control.writeValueWithResponse = bytes => bytes[0] === 3 ? Promise.reject(new Error('Write failed')) : write(bytes);
  session.setNetwork('Home'); await flush();
  assert.equal(session.phase, 'disconnected');
  assert.match(session.networkMessage, /not confirmed/i); noMotion(fake);
});

test('stalled command is bounded and a late completion cannot restore the session', async () => {
  const { fake, session, advance } = await ready(), gate = pending();
  const write = fake.chars.control.writeValueWithResponse;
  fake.chars.control.writeValueWithResponse = bytes => bytes[0] === 3 ? gate.promise.then(() => write(bytes)) : write(bytes);
  session.setNetwork('Home'); await flush();
  for (let n = 0; n < 20; n++) await advance(50);
  assert.equal(session.phase, 'disconnected');
  gate.resolve(); await flush();
  assert.equal(session.phase, 'disconnected'); assert.equal(session.localArm, false); noMotion(fake);
});

test('invalid metadata disables network actions and old callbacks cannot update a new generation', async () => {
  const { fake, session } = await ready();
  fake.notifyNetwork([255]); await flush();
  assert.equal(session.networkInfo, null); assert.equal(session.phase, 'ready');
  session.setNetwork('Home'); await flush(); assert.deepEqual(modeWrites(fake), []);
  fake.notifyNetwork([2, 0, 0, 0, 192, 168, 1, 193]); await flush();
  assert.equal(session.networkInfo.mode, 'Home'); assert.equal(session.networkProblem, '');
  await session.close(); const info = session.networkInfo;
  fake.notifyNetwork([2, 0, 0, 0, 1, 2, 3, 4]); await flush();
  assert.equal(session.networkInfo, info);
});
