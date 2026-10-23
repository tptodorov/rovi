import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Ble, pickDevice } from '../transport.js';
import { SERVICE, status, setpoint } from '../wire.js';
import { fakeBle, pending } from './fake-ble.js';

test('picker uses name-only advertisement and optional service without claiming', async () => {
  const fake = fakeBle();
  assert.equal(await pickDevice(fake.bluetooth, 'https:'), fake.device);
  assert.deepEqual(fake.calls, [['picker', { filters: [{ name: 'Rovi M8' }], optionalServices: [SERVICE] }]]);
  assert.throws(() => pickDevice(fake.bluetooth, 'http:'), /HTTPS/);
  assert.throws(() => pickDevice({}, 'https:'), /Bluefy/);
});

test('normal GATT contract uses exact writes and explicit reads separately from notifications', async () => {
  const fake = fakeBle(), reports = [];
  const ble = new Ble(fake.device, bytes => reports.push(status(bytes)), () => {});
  const initial = await ble.open();
  assert.equal(initial.caps.length, 27); assert.equal(status(initial.report).state, 'Stopped', 'claimed by the STOP that open() sends');
  await ble.send(setpoint(0)); await ble.control(1);
  assert.equal(reports.at(-1).state, 'Armed'); assert.equal(status(await ble.read()).state, 'Armed');
  await ble.control(2); assert.equal(status(await ble.read()).reason, 'Stop');
  ble.disconnect(); fake.notify([2, 0, 0]); assert.equal(reports.at(-1).state, 'Stopped');
});

test('claim-on-write firmware is claimed with an acknowledged STOP before Status is read', async () => {
  const fake = fakeBle({ claimOnWrite: true });
  const ble = new Ble(fake.device, () => {}, () => {});
  const initial = await ble.open();
  assert.deepEqual(status(initial.report), { state: 'Stopped', reason: 'Stop', error: 'none' });
  const control = fake.calls.findIndex(([kind, code]) => kind === 'control' && code === 2);
  assert.ok(control >= 0 && control < fake.calls.findIndex(([kind]) => kind === 'read'), fake.calls);
  assert.equal(fake.calls.some(([kind, code]) => kind === 'setpoint' || (kind === 'control' && code === 1)), false);
});

test('a car another controller owns is reported busy, never mistaken for a claim', async () => {
  const fake = fakeBle({ claimOnWrite: true, busy: true });
  const ble = new Ble(fake.device, () => {}, () => {});
  await assert.rejects(ble.open(), /another controller owns the car/);
  assert.equal(fake.calls.some(([kind]) => kind === 'setpoint'), false);
});

test('incompatible properties or missing split write APIs fail before any setpoint/control', async () => {
  for (const mutate of [f => { f.chars.setpoint.properties = {}; }, f => { delete f.chars.control.writeValueWithResponse; }, f => { f.chars.status.properties.notify = false; }]) {
    const fake = fakeBle(); mutate(fake);
    const ble = new Ble(fake.device, () => {}, () => {});
    await assert.rejects(ble.open(), /Unsupported/);
    assert.equal(fake.calls.some(([kind]) => kind === 'setpoint' || kind === 'control'), false);
    ble.disconnect();
  }
});

test('late connect completion after cancellation disconnects again without discovering/writing', async () => {
  const fake = fakeBle(), connect = pending();
  fake.server.connect = () => connect.promise;
  const ble = new Ble(fake.device, () => {}, () => {}), opening = ble.open();
  ble.disconnect(); fake.server.connected = true; connect.resolve(fake.server);
  await assert.rejects(opening, /cancellation/);
  assert.equal(fake.server.connected, false);
  assert.deepEqual(fake.calls.map(x => x[0]), ['disconnect', 'disconnect']);
});
