// The WebAssembly binding against crates/rovi-protocol/vectors.json (add-rovi-protocol task 4.1):
// the same results as the Rust crate and the Python binding.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import init, * as rp from '../rovi_protocol.js';

await init({ module_or_path: readFileSync(new URL('../rovi_protocol_bg.wasm', import.meta.url)) });
const VECTORS = JSON.parse(readFileSync(new URL('../../../crates/rovi-protocol/vectors.json', import.meta.url), 'utf8'));
const bytes = hex => Uint8Array.from(Buffer.from(hex, 'hex'));
const hex = b => Buffer.from(b).toString('hex');
const FLOATS = ['wheels_radius', 'center_projection_sum', 'max_vx', 'max_vy', 'max_wz'];
const caps = v => Object.fromEntries(Object.entries(v).map(([k, x]) => [k, FLOATS.includes(k) ? Math.fround(x) : x]));
const same = v => v;
const CODECS = {
  'status binary': [rp.statusDecode, rp.statusEncode, same],
  'status ros': [rp.statusDecodeRos, rp.statusEncodeRos, same],
  'capabilities binary': [rp.capabilitiesDecode, rp.capabilitiesEncode, caps],
  'capabilities ros': [rp.capabilitiesDecodeRos, rp.capabilitiesEncodeRos, caps],
  'network binary': [rp.networkDecode, rp.networkEncode, same],
  'network ros': [rp.networkDecodeRos, rp.networkEncodeRos, same],
  'setpoint cdr': [rp.setpointDecode, rp.setpointEncode, same],
};
// The direction a UDP frame travels, from its kind and payload.
const fromCar = b => [2, 7, 8].includes(b[3]) || (b[3] === 9 && b.length > 12 + 32) || (b[3] === 11 && b.length > 12);

test('every valid vector decodes to its value and encodes back to its bytes', () => {
  const seen = new Set();
  for (const v of VECTORS.valid) {
    const key = `${v.message} ${v.encoding}`, b = bytes(v.hex), label = `${key} ${v.name}`;
    seen.add(key);
    if (CODECS[key]) {
      const [decode, encode, expect] = CODECS[key];
      assert.deepEqual(decode(b), expect(v.value), label);
      assert.equal(hex(encode(v.value)), v.hex, label);
    } else if (key === 'setpoint cdr-decode-only') {
      assert.deepEqual(rp.setpointDecode(b), v.value, label);
    } else if (key === 'ble-setpoint binary') {
      assert.deepEqual(rp.bleSetpointDecode(b), v.value, label);
      assert.equal(hex(rp.bleSetpointEncode(v.value.seq, v.value.setpoint)), v.hex, label);
    } else if (key === 'ble-control binary') {
      assert.equal(rp.controlDecode(b), v.value, label);
      assert.equal(hex(rp.controlEncode(v.value)), v.hex, label);
    } else if (key === 'udp-frame binary') {
      const [decode, encode] = fromCar(b) ? [rp.udpDecodeFromCar, rp.udpEncodeFromCar] : [rp.udpDecodeToCar, rp.udpEncodeToCar];
      const frame = decode(b);
      assert.deepEqual([frame.session, frame.seq], [v.value.session, v.value.sequence], label);
      assert.equal(hex(encode(frame)), v.hex, label);
    } else assert.fail(`no binding check for ${key}`);
  }
  assert.equal(seen.size, Object.keys(CODECS).length + 4);
});

test('every malformed vector raises DecodeError with the crate\'s kind', () => {
  const decoders = { status: rp.statusDecode, capabilities: rp.capabilitiesDecode, network: rp.networkDecode, setpoint: rp.setpointDecode };
  assert.ok(VECTORS.invalid.length >= 40);
  for (const v of VECTORS.invalid) {
    const b = bytes(v.hex), decode = decoders[v.message] ?? (fromCar(b) ? rp.udpDecodeFromCar : rp.udpDecodeToCar);
    assert.throws(() => decode(b), e => e.name === 'DecodeError' && e.kind === v.error, `${v.message} ${v.name}`);
  }
});

test('encoders refuse what decoders refuse, and sequences are never wrapped', () => {
  assert.throws(() => rp.setpointEncode({ vx: NaN }), e => e.name === 'EncodeError');
  assert.throws(() => rp.statusEncode({ state: 'stopped', reason: 'none', error: 'none' }), e => e.name === 'EncodeError' && e.kind === 'inconsistent');
  for (const seq of [-1, 0.5, NaN, 2 ** 32]) assert.throws(() => rp.bleSetpointEncode(seq, {}), e => e.kind === 'invalid', String(seq));
  assert.equal(rp.constants().minLeaseMs, 500);
  assert.equal(rp.constants().bleUuids.network, 'd47a0016-45b2-4d19-8db0-6bd87e9c0001');
});
