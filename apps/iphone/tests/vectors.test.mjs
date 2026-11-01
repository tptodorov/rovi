// The phone's wire adapter against the protocol vectors; protocol.test.mjs covers the whole module.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { capabilities, network, setpoint, status } from '../wire.js';

const VECTORS = JSON.parse(readFileSync(new URL('../../../crates/rovi-protocol/vectors.json', import.meta.url), 'utf8'));
const bytes = hex => Uint8Array.from(Buffer.from(hex, 'hex'));
const cases = (kind, message, encoding) => VECTORS[kind].filter(v => v.message === message && (!encoding || v.encoding === encoding));

test('valid Status, Capabilities and Network decode to the vector values', () => {
  for (const v of cases('valid', 'status', 'binary')) {
    assert.deepEqual(status(bytes(v.hex)), { state: v.value.state, reason: v.value.reason, error: v.value.error }, v.name);
  }
  for (const v of cases('valid', 'capabilities', 'binary')) {
    const caps = capabilities(bytes(v.hex)), want = v.value;
    assert.equal(caps.version, want.version);
    for (const [key, name] of [['radius', 'wheels_radius'], ['projection', 'center_projection_sum'], ['max_vx'], ['max_vy'], ['max_wz']]) {
      assert.equal(caps[key], Math.fround(want[name ?? key]), `${v.name} ${key}`);
    }
    for (const key of ['lease_ms', 'claim_ms', 'reclaim_ms']) assert.equal(caps[key], want[key]);
  }
  for (const v of cases('valid', 'network', 'binary')) {
    assert.deepEqual(network(bytes(v.hex)), v.value, v.name);
  }
});

test('BLE setpoints encode to the vector bytes', () => {
  for (const v of cases('valid', 'ble-setpoint')) {
    const { seq, setpoint: s } = v.value;
    assert.equal(Buffer.from(setpoint(seq, s)).toString('hex'), v.hex, v.name);
  }
});

test('malformed Status, Capabilities and Network are rejected', () => {
  for (const message of ['status', 'capabilities', 'network']) {
    const decode = { status, capabilities, network }[message];
    for (const v of cases('invalid', message, 'binary')) assert.throws(() => decode(bytes(v.hex)), `${message} ${v.name}`);
  }
});
