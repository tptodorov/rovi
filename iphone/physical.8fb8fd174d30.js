import { Hold } from './hold.8fb8fd174d30.js';
import { ZERO } from './wire.8fb8fd174d30.js';
import { angleDelta, scaleAngle } from './angles.8fb8fd174d30.js';

const finite = value => typeof value === 'number' && Number.isFinite(value);
export class Physical {
  hold = new Hold();
  enabled = false;
  sample = null;
  neutral = null;
  problem = null;

  clear() { this.hold.clear(); }
  invalidate(reason) { this.neutral = null; this.problem = reason; this.clear(); }
  enable() { this.sample = null; this.enabled = true; this.invalidate('Wait for fresh readings, then calibrate neutral.'); }
  disable(reason = 'Enable sensors and calibrate before ARM.') {
    this.enabled = false; this.sample = null; this.invalidate(reason);
  }
  fresh(now) { return this.enabled && this.sample && now >= this.sample.at && now - this.sample.at < 250; }

  update(value, now) {
    if (!this.enabled) return;
    const { alpha, beta, gamma } = value ?? {};
    if (![alpha, beta, gamma].every(finite) || alpha < 0 || alpha >= 360 || beta < -180 || beta >= 180 || gamma < -90 || gamma >= 90) {
      this.sample = null; this.invalidate('Invalid sensor reading; enable/calibrate again.'); return;
    }
    const reference = typeof value.absolute === 'boolean' ? value.absolute : null;
    if (this.neutral && (!this.fresh(now) || reference !== this.sample.reference)) {
      this.invalidate('Sensor gap or reference changed; calibrate again.');
    }
    this.sample = { alpha, beta, gamma, reference, at: now };
    this.ready(now);
  }

  calibrate(now) {
    this.clear();
    if (!this.fresh(now)) { this.invalidate('Fresh sensor readings are required to calibrate.'); return false; }
    if (Math.abs(this.sample.beta) > 30 || Math.abs(this.sample.gamma) > 30) {
      this.invalidate('Hold the phone screen up, within 30° of flat, then calibrate.'); return false;
    }
    this.neutral = { ...this.sample }; this.problem = null; return true;
  }

  ready(now) {
    if (this.neutral && !this.fresh(now)) this.invalidate('Sensor readings expired; reconnect and calibrate.');
    if (this.sample && (Math.abs(this.sample.beta) > 60 || Math.abs(this.sample.gamma) > 60 ||
        (this.neutral && Math.abs(angleDelta(this.sample.alpha, this.neutral.alpha)) > 90))) {
      this.invalidate('Phone moved outside the calibrated operating posture.');
    }
    if (this.hold.current !== null && now >= this.hold.expires) this.invalidate('Drive hold expired; reconnect and calibrate.');
    return Boolean(this.neutral && !this.problem && this.fresh(now));
  }

  press(id, allowed, now) { this.hold.track(id, 'drive', allowed && this.ready(now), now); }
  release(id) { this.hold.release(id); }
  renew(now) { if (this.ready(now)) this.hold.renew(now); }

  intent(caps, now) {
    if (!this.ready(now) || this.hold.current === null) return ZERO;
    const result = {};
    for (const [axis, angle, sign, gentle] of [['vx', 'beta', -1, .1], ['vy', 'gamma', -1, .1], ['wz', 'alpha', 1, .3]]) {
      const maximum = caps?.[`max_${axis}`];
      result[axis] = finite(maximum) && maximum > 0 ?
        scaleAngle(sign * angleDelta(this.sample[angle], this.neutral[angle])) * Math.min(gentle, maximum) || 0 : 0;
    }
    return result;
  }
}
