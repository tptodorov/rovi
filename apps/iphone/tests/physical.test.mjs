import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Physical } from '../physical.js';
import { ZERO } from '../wire.js';

const caps = { max_vx: 1, max_vy: 1, max_wz: 3 };
const neutral = { alpha: 350, beta: 0, gamma: 0, absolute: false };
const tilt = { alpha: 10, beta: -20, gamma: -20, absolute: false };
function calibrated() {
  const input = new Physical(); input.enable(); input.update(neutral, 0);
  assert.equal(input.calibrate(0), true); return input;
}
test('explicit start at neutral gates combined capped axes and heading wrap', () => {
  const input = new Physical(); input.update(neutral, 0); assert.equal(input.start(0), false);
  input.enable(); input.update(neutral, 0); input.calibrate(0);
  input.update(tilt, 1); assert.equal(input.start(1), false);
  assert.deepEqual(input.intent(caps, 1), ZERO);
  input.update(neutral, 2); assert.deepEqual(input.intent(caps, 2), ZERO);
  assert.equal(input.start(2), true); input.update(tilt, 3);
  assert.deepEqual(input.intent(caps, 3), { vx: .1, vy: .1, wz: .3 });
  assert.deepEqual(input.intent({ max_vx: .05, max_vy: 0, max_wz: .2 }, 3), { vx: .05, vy: 0, wz: .2 });
  input.update({ ...neutral, alpha: 330, beta: 20, gamma: 20 }, 4);
  assert.deepEqual(input.intent(caps, 4), { vx: -.1, vy: -.1, wz: -.3 });
  input.clear(); assert.deepEqual(input.intent(caps, 4), ZERO);
  assert.notEqual(input.neutral, null);
});
test('fresh unchanged angles sustain twist; late samples cannot hide expiry or revive driving', () => {
  const input = calibrated(); input.start(1);
  for (let at = 20; at <= 1000; at += 20) {
    input.update(tilt, at); input.renew(at); assert.equal(input.intent(caps, at).wz, .3);
  }
  input.renew(1200); assert.deepEqual(input.intent(caps, 1250), ZERO);
  input.update(tilt, 1251); input.renew(1251); assert.deepEqual(input.intent(caps, 1251), ZERO);
  const betweenTicks = calibrated(); betweenTicks.start(1); betweenTicks.renew(240);
  betweenTicks.update(tilt, 260); assert.deepEqual(betweenTicks.intent(caps, 260), ZERO);
  assert.equal(betweenTicks.driving, false);
});
test('independent foreground expiry and pause cannot resume from fresh data or UI renewal', () => {
  const input = calibrated(); input.start(1);
  for (const at of [100, 200, 250]) {
    input.update(neutral, at); assert.equal(input.start(at), false);
  }
  assert.deepEqual(input.intent(caps, 251), ZERO); assert.equal(input.neutral, null);
  assert.match(input.problem, /Foreground renewal expired/);
  input.update(neutral, 252); input.calibrate(252); assert.equal(input.start(252), true);
  input.clear(); input.update(tilt, 253); input.renew(253);
  assert.deepEqual(input.intent(caps, 253), ZERO); assert.equal(input.start(253), false);
  input.update(neutral, 254); input.renew(254); assert.deepEqual(input.intent(caps, 254), ZERO);
});
test('invalid axes, changed reference and operating limits stop driving with precise reasons', () => {
  for (const value of [null, NaN, Infinity, '0', -1, 360]) {
    const input = calibrated(); input.start(1); input.update({ ...tilt, alpha: value }, 2);
    assert.equal(input.neutral, null); assert.deepEqual(input.intent(caps, 2), ZERO);
  }
  for (const [value, reason] of [
    [{ ...tilt, absolute: true }, /reference changed/],
    [{ ...tilt, beta: 61 }, /Forward\/back tilt β 61.0° exceeds ±60°/],
    [{ ...tilt, gamma: -61 }, /Side tilt γ -61.0° exceeds ±60°/],
    [{ ...tilt, alpha: 100 }, /Rotation offset 110.0° exceeds ±90°/],
  ]) {
    const input = calibrated(); input.start(1); input.update(value, 2);
    assert.equal(input.neutral, null); assert.deepEqual(input.intent(caps, 2), ZERO);
    assert.match(input.problem, reason);
  }
  const input = new Physical(); input.enable(); input.update({ ...neutral, beta: 31 }, 0);
  assert.equal(input.calibrate(0), false);
});
test('START requires all three offsets within 3 degrees, including wrap', () => {
  for (const axis of ['alpha', 'beta', 'gamma']) {
    const input = calibrated(); input.update({ ...neutral, [axis]: (neutral[axis] + 3.1) % 360 }, 1);
    assert.equal(input.start(1), false, axis); assert.equal(input.driving, false);
  }
  const input = calibrated(); input.update({ ...neutral, alpha: 353, beta: -3, gamma: 3 }, 1);
  assert.equal(input.start(1), true);
  assert.deepEqual(input.intent(caps, 1), ZERO);
  input.clear(); input.update(neutral, 251); assert.equal(input.start(501), false);
});
