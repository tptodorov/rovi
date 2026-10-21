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
test('explicit calibration and fresh permitted press gate combined capped axes and heading wrap', () => {
  const input = new Physical(); input.update(neutral, 0); assert.equal(input.calibrate(0), false);
  input.enable(); input.update(neutral, 0); input.press(1, false, 0); input.calibrate(0);
  input.update(tilt, 1); assert.deepEqual(input.intent(caps, 1), ZERO);
  input.press(2, true, 1); assert.deepEqual(input.intent(caps, 1), ZERO);
  input.release(1); input.release(2); input.press(3, true, 2);
  assert.deepEqual(input.intent(caps, 2), { vx: .1, vy: .1, wz: .3 });
  assert.deepEqual(input.intent({ max_vx: .05, max_vy: 0, max_wz: .2 }, 2), { vx: .05, vy: 0, wz: .2 });
  input.update({ ...neutral, alpha: 330, beta: 20, gamma: 20 }, 3);
  assert.deepEqual(input.intent(caps, 3), { vx: -.1, vy: -.1, wz: -.3 });
  input.release(3); assert.deepEqual(input.intent(caps, 3), ZERO);
});
test('fresh unchanged angles sustain twist; cached values and late samples cannot revive it', () => {
  const input = calibrated(); input.press(1, true, 1);
  for (let at = 20; at <= 1000; at += 20) {
    input.update(tilt, at); input.renew(at); assert.equal(input.intent(caps, at).wz, .3);
  }
  input.renew(1200); assert.deepEqual(input.intent(caps, 1250), ZERO);
  input.update(tilt, 1251); input.renew(1251); assert.deepEqual(input.intent(caps, 1251), ZERO);
  const betweenTicks = calibrated(); betweenTicks.press(1, true, 1); betweenTicks.renew(240);
  betweenTicks.update(tilt, 260); assert.deepEqual(betweenTicks.intent(caps, 260), ZERO);
});
test('independent UI expiry, multi-touch and cancellation cannot reuse an old pointer', () => {
  const input = calibrated(); input.press(1, true, 1);
  for (const at of [100, 200, 250]) input.update(tilt, at);
  assert.deepEqual(input.intent(caps, 251), ZERO); assert.equal(input.neutral, null);
  input.calibrate(251); input.release(1); input.press(2, true, 252); input.press(3, true, 252);
  assert.deepEqual(input.intent(caps, 252), ZERO);
  input.release(3); input.renew(253); assert.deepEqual(input.intent(caps, 253), ZERO);
  input.release(2); input.press(4, true, 254); input.clear(); input.renew(254);
  assert.deepEqual(input.intent(caps, 254), ZERO);
});
test('invalid axes, changed reference and unsafe postures revoke calibration', () => {
  for (const value of [null, NaN, Infinity, '0', -1, 360]) {
    const input = calibrated(); input.press(1, true, 1); input.update({ ...tilt, alpha: value }, 2);
    assert.equal(input.neutral, null); assert.deepEqual(input.intent(caps, 2), ZERO);
  }
  for (const value of [{ ...tilt, absolute: true }, { ...tilt, beta: 61 }, { ...tilt, gamma: -61 }, { ...tilt, alpha: 100 }]) {
    const input = calibrated(); input.press(1, true, 1); input.update(value, 2);
    assert.equal(input.neutral, null); assert.deepEqual(input.intent(caps, 2), ZERO);
  }
  const input = new Physical(); input.enable(); input.update({ ...neutral, beta: 31 }, 0);
  assert.equal(input.calibrate(0), false);
});
