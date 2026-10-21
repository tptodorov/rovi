import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { capabilities, status, setpoint, direction, DIRECTIONS } from '../wire.js';

export function capsBytes() {
  return Uint8Array.from(Buffer.from('018fc2f53c5c8f023e3333333f9a99993e00004040e80310278813', 'hex'));
}

test('BLE setpoints match the existing Python wire codec, including wrap and every direction', () => {
  const caps = capabilities(capsBytes());
  const samples = [0xffffffff, 0, 1].flatMap(seq => Object.keys(DIRECTIONS).map(name => [seq, direction(caps, name)]));
  samples.push([0, { vx: 0, vy: 0, wz: 0 }]);
  const reference = JSON.parse(execFileSync('python3', ['-c', `
import sys, json
sys.path.insert(0, '../../experiments/rovi-m8-setpoint-ros/sim')
import wire
samples = json.loads(sys.stdin.read())
print(json.dumps([wire.ble_setpoint(seq, **motion).hex() for seq, motion in samples]))
`], { cwd: new URL('..', import.meta.url), input: JSON.stringify(samples), encoding: 'utf8' }));
  samples.forEach(([seq, motion], i) => assert.equal(Buffer.from(setpoint(seq, motion)).toString('hex'), reference[i]));
  assert.throws(() => setpoint(0, { vx: NaN }));
  assert.throws(() => setpoint(0x100000000));
});

test('capabilities preserve canonical f32 values and enforce supported profiles', () => {
  const caps = capabilities(capsBytes());
  assert.equal(caps.max_vx, Math.fround(0.7));
  assert.equal(caps.lease_ms, 1000);
  for (const [offset, value] of [[1, NaN], [5, 0], [9, -1], [13, Infinity]]) {
    const bytes = capsBytes(); new DataView(bytes.buffer).setFloat32(offset, value, true);
    assert.throws(() => capabilities(bytes));
  }
  for (const [offset, value] of [[21, 499], [23, 999], [25, 0]]) {
    const bytes = capsBytes(); new DataView(bytes.buffer).setUint16(offset, value, true);
    assert.throws(() => capabilities(bytes));
  }
  assert.throws(() => capabilities(capsBytes().slice(1)));
  const version = capsBytes(); version[0] = 2; assert.throws(() => capabilities(version));
  assert.equal(direction({ max_vx: 0.05 }, 'forward').vx, 0.05);
  assert.equal(direction({ max_vy: 0 }, 'left').vy, 0);
});

test('Status validates all codes and rejects inconsistent state/reason pairs', () => {
  for (let state = 0; state < 4; state++) for (let reason = 0; reason < 5; reason++) for (let error = 0; error < 5; error++) {
    const bytes = Uint8Array.of(state, reason, error);
    if (state === 3 ? reason !== 0 : reason === 0) assert.ok(status(bytes).state);
    else assert.throws(() => status(bytes));
  }
  for (const bytes of [[4, 0, 0], [3, 5, 0], [1, 0, 5], [1, 0]]) assert.throws(() => status(Uint8Array.from(bytes)));
  assert.deepEqual(status(Uint8Array.of(3, 2, 0)), { state: 'Stopped', reason: 'LeaseExpired', error: 'none' });
});
