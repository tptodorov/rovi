import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Hold } from '../hold.js';
import { ZERO, DIRECTIONS } from '../wire.js';

const caps = { max_vx: 0.05, max_vy: 0.1, max_wz: 0.2 };
test('all directions use one capped axis; no press can arm', () => {
  for (const [name, [axis, sign]] of Object.entries(DIRECTIONS)) {
    const hold = new Hold(); hold.press(1, name, false, 0);
    assert.deepEqual(hold.intent(caps, 1), ZERO);
    hold.release(1); hold.press(2, name, true, 2);
    assert.deepEqual(hold.intent(caps, 3), { ...ZERO, [axis]: sign * caps[`max_${axis}`] });
  }
});
test('250 ms expiry invalidates even a later UI renewal until a new press', () => {
  const hold = new Hold(); hold.press(1, 'forward', true, 0);
  assert.equal(hold.intent(caps, 249).vx, 0.05);
  assert.deepEqual(hold.intent(caps, 250), ZERO);
  hold.renew(251); assert.deepEqual(hold.intent(caps, 251), ZERO);
  hold.release(1); hold.press(2, 'forward', true, 252);
  hold.renew(400); assert.equal(hold.intent(caps, 500).vx, 0.05);
  hold.renew(650); assert.deepEqual(hold.intent(caps, 650), ZERO);
});
test('release, cancellation and leaving bounds never resume an old touch', () => {
  const hold = new Hold(); hold.press(1, 'left', true, 0);
  hold.clear(); hold.renew(1); assert.deepEqual(hold.intent(caps, 2), ZERO);
  hold.release(1); hold.press(2, 'right', true, 3);
  assert.equal(hold.intent(caps, 4).vy, -0.1);
  hold.release(2); assert.deepEqual(hold.intent(caps, 5), ZERO);
});
test('multi-touch and ARM/STOP invalidation require all old fingers released', () => {
  const hold = new Hold(); hold.press(1, 'forward', true, 0);
  hold.press(2, 'left', true, 1); assert.deepEqual(hold.intent(caps, 2), ZERO);
  hold.release(2); hold.renew(3); assert.deepEqual(hold.intent(caps, 4), ZERO);
  hold.press(3, 'cw', true, 5); assert.deepEqual(hold.intent(caps, 6), ZERO);
  hold.release(1); hold.release(3); hold.press(4, 'backward', true, 7);
  assert.equal(hold.intent(caps, 8).vx, -0.05);
  hold.clear(); assert.deepEqual(hold.intent(caps, 9), ZERO);
});
