import { test } from 'node:test';
import assert from 'node:assert/strict';
import { SensorProbe, Cadence, angleDelta, scaleAngle } from '../sensor-model.js';

const zero = { forward: 0, left: 0, rotateLeft: 0 };
const origin = { alpha: 350, beta: 10, gamma: 5 };
const tilted = { alpha: 10, beta: -10, gamma: -15 };
function ready() {
  const probe = new SensorProbe(); probe.enable(0); probe.record('orientation', origin, 0);
  assert.equal(probe.calibrate(0), true); return probe;
}
test('three axes combine around calibrated neutral; yaw crosses 0/360 in either direction', () => {
  const probe = ready(); probe.press(1); probe.record('orientation', tilted, 2);
  assert.deepEqual(probe.consume(2), { forward: 1, left: 1, rotateLeft: 1 });
  probe.record('orientation', origin, 3); assert.deepEqual(probe.consume(3), zero);
  assert.equal(angleDelta(350, 10), -20); assert.equal(angleDelta(10, 350), 20);
  assert.equal(scaleAngle(3), 0); assert.equal(scaleAngle(-3), 0);
  assert.equal(scaleAngle(100), 1); assert.equal(scaleAngle(-100), -1);
});
test('steady new readings sustain the angle; release and cached readings cannot', () => {
  const probe = ready(); probe.press(1);
  for (let at = 20; at <= 1000; at += 20) {
    probe.record('orientation', tilted, at); probe.frame(at);
    assert.equal(probe.consume(at).rotateLeft, 1);
  }
  probe.release(); assert.deepEqual(probe.consume(1001), zero);
  probe.press(1002); probe.frame(1200);
  assert.deepEqual(probe.consume(1250), zero);
  probe.record('orientation', tilted, 1251); probe.frame(1251);
  assert.deepEqual(probe.consume(1251), zero);
  assert.equal(probe.neutral, null);
});
test('a delayed sample cannot hide a missed cutoff between consumer ticks', () => {
  const probe = ready(); probe.press(1); probe.frame(240);
  probe.record('orientation', tilted, 260);
  assert.deepEqual(probe.consume(260), zero); assert.equal(probe.neutral, null);
});
test('fresh sensors cannot renew an expired UI hold; re-enabling requires new samples and calibration', () => {
  const probe = ready(); probe.press(1);
  for (const at of [100, 200, 250]) probe.record('orientation', tilted, at);
  assert.deepEqual(probe.consume(251), zero);
  probe.frame(252); assert.deepEqual(probe.consume(252), zero);
  probe.disable('hidden', 253); probe.enable(254);
  assert.equal(probe.calibrate(254), false);
  probe.record('orientation', origin, 255); assert.equal(probe.calibrate(255), true);
  assert.deepEqual(probe.consume(255), zero);
});
test('missing/non-finite/coerced fields revoke calibration and never become valid zeroes', () => {
  for (const value of [null, undefined, NaN, Infinity, '0']) {
    const probe = ready(); probe.press(1);
    probe.record('orientation', { ...tilted, alpha: value }, 2);
    assert.deepEqual(probe.consume(2), zero); assert.equal(probe.neutral, null);
    assert.equal(probe.fields['orientation.alpha'].invalid, 1);
  }
});
test('cadence reports measured arrival gaps; disabled consumers do not extend finished runs', () => {
  const cadence = new Cadence(); for (const at of [0, 20, 40, 100]) cadence.record(at);
  assert.deepEqual(cadence.summary(125), { count: 4, durationMs: 100, ageMs: 25,
    maxGapMs: 60, recentGapCount: 3, medianGapMs: 20, p95GapMs: 60 });
  const probe = ready(); probe.consume(1); probe.disable('finished', 2); probe.consume(1000);
  assert.equal(probe.report(1000).cadences.consumer.count, 1);
});
test('saved stationary drift/jitter excludes later deliberate tilt and survives interruptions', () => {
  const probe = ready();
  probe.record('orientation', { ...origin, alpha: 351, beta: 9.5 }, 20);
  probe.record('orientation', { ...origin, alpha: 349, beta: 10.5 }, 40);
  assert.equal(probe.saveStationary(40), true);
  probe.record('orientation', tilted, 60); probe.disable('finished', 80);
  const [result] = probe.report(80).stationaryResults;
  assert.equal(result.lastAtMs - result.startedAtMs, 40); assert.equal(result.count, 2);
  assert.equal(result.minDeltaDegrees.alpha, -1); assert.equal(result.maxDeltaDegrees.alpha, 1);
  assert.equal(result.lastDeltaDegrees.beta, .5);
});
