import { ZERO } from './wire.cef9e9bca8fe.js';
import { angleDelta, scaleAngle } from './angles.cef9e9bca8fe.js';

const finite = value => typeof value === 'number' && Number.isFinite(value);
export class Physical {
  driving = false;
  expires = 0;
  enabled = false;
  sample = null;
  neutral = null;
  problem = null;

  clear() { this.driving = false; this.expires = 0; }
  invalidate(reason) { this.neutral = null; this.problem = reason; this.clear(); }
  enable() { this.sample = null; this.enabled = true; this.invalidate('Hold neutral, then tap Set neutral & ARM.'); }
  disable(reason = 'Enable sensors, then Set neutral & ARM.') {
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
    if (this.neutral && ((!this.fresh(now) && this.driving) || reference !== this.sample.reference)) {
      this.invalidate('Sensor gap during drive or reference changed; connect and ARM again.');
    }
    this.sample = { alpha, beta, gamma, reference, at: now };
    this.ready(now);
  }

  canCalibrate(now) {
    return Boolean(this.fresh(now) && Math.abs(this.sample.beta) <= 30 && Math.abs(this.sample.gamma) <= 30);
  }

  calibrate(now) {
    this.clear();
    if (!this.fresh(now)) { this.invalidate('Fresh sensor readings are required to calibrate.'); return false; }
    if (!this.canCalibrate(now)) {
      this.invalidate('Hold the phone screen up, within 30° of flat, then calibrate.'); return false;
    }
    this.neutral = { ...this.sample }; this.problem = null; return true;
  }

  ready(now) {
    const yaw = this.neutral && this.sample ? angleDelta(this.sample.alpha, this.neutral.alpha) : 0;
    const outside = this.sample && (Math.abs(this.sample.beta) > 60 ? `Forward/back tilt β ${this.sample.beta.toFixed(1)}° exceeds ±60°` :
      Math.abs(this.sample.gamma) > 60 ? `Side tilt γ ${this.sample.gamma.toFixed(1)}° exceeds ±60°` :
      Math.abs(yaw) > 90 ? `Rotation offset ${yaw.toFixed(1)}° exceeds ±90°` : null);
    // Paused control stays at zero; reading/scrolling must not discard neutral.
    if (this.driving) {
      if (!this.fresh(now)) this.invalidate('Sensor readings expired during drive; connect and ARM again.');
      else if (outside) this.invalidate(`${outside}. Tilt control stopped; connect and ARM again.`);
    }
    if (this.driving && now >= this.expires) this.invalidate('Foreground renewal expired; connect and ARM again.');
    return Boolean(this.neutral && !this.problem && this.fresh(now) && !outside);
  }

  canStart(now) {
    return this.ready(now) && ['alpha', 'beta', 'gamma'].every(axis => Math.abs(angleDelta(this.sample[axis], this.neutral[axis])) <= 3);
  }
  start(now) {
    if (this.driving || !this.canStart(now)) return false;
    this.driving = true; this.expires = now + 250; return true;
  }
  renew(now) { if (this.ready(now) && this.driving) this.expires = now + 250; }

  intent(caps, now) {
    if (!this.ready(now) || !this.driving) return ZERO;
    const result = {};
    for (const [axis, angle, sign, gentle] of [['vx', 'beta', -1, .1], ['vy', 'gamma', -1, .1], ['wz', 'alpha', 1, .3]]) {
      const maximum = caps?.[`max_${axis}`];
      result[axis] = finite(maximum) && maximum > 0 ?
        scaleAngle(sign * angleDelta(this.sample[angle], this.neutral[angle])) * Math.min(gentle, maximum) || 0 : 0;
    }
    return result;
  }
}
