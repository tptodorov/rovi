// Diagnostic only. This module has no session, transport or command encoder.
import { angleDelta, scaleAngle } from './angles.82fbd45f4050.js';
export { angleDelta, scaleAngle };
export const STALE_MS = 250;
const finite = value => typeof value === 'number' && Number.isFinite(value);
const relative = (value, neutral) => Object.fromEntries(['alpha', 'beta', 'gamma'].map(key => [key, angleDelta(value[key], neutral[key])]));

export class Cadence {
  count = 0; first = null; last = null; maximum = 0; gaps = [];
  record(now) {
    if (this.last !== null) {
      const gap = now - this.last;
      this.maximum = Math.max(this.maximum, gap);
      this.gaps.push(gap);
      if (this.gaps.length > 4096) this.gaps.shift();
    }
    this.first ??= now; this.last = now; this.count++;
  }
  summary(now) {
    const sorted = [...this.gaps].sort((a, b) => a - b);
    const percentile = p => sorted.length ? sorted[Math.ceil(sorted.length * p) - 1] : null;
    return { count: this.count, durationMs: this.last === null ? 0 : this.last - this.first,
      ageMs: this.last === null ? null : now - this.last, maxGapMs: this.maximum,
      recentGapCount: sorted.length, medianGapMs: percentile(.5), p95GapMs: percentile(.95) };
  }
}

export class SensorProbe {
  enabled = false; neutral = null; held = false; holdAt = null;
  orientation = null; motion = null; reason = 'Enable sensors to begin';
  cadences = Object.fromEntries(['orientation', 'motion', 'ui', 'consumer'].map(name => [name, new Cadence()]));
  fields = {}; events = []; samples = []; enabledAt = null;
  stationary = null; stationaryResults = [];
  log(type, now) { this.events.push({ type, atMs: now }); if (this.events.length > 80) this.events.shift(); }
  invalidate(reason, now) {
    this.neutral = null; this.stationary = null; this.held = false; this.holdAt = null; this.reason = reason;
    this.log(reason, now);
  }
  enable(now) {
    this.enabled = true; this.enabledAt = now; this.orientation = null; this.motion = null;
    this.invalidate('Waiting for fresh orientation; calibrate at neutral', now);
  }
  disable(reason, now) { this.enabled = false; this.invalidate(reason, now); }
  record(kind, values, now) {
    if (!this.enabled) return;
    if (kind === 'orientation' && this.neutral && now - this.orientation.at >= STALE_MS) {
      this.invalidate('Orientation gap exceeded cutoff; recalibrate', now);
    }
    this.cadences[kind].record(now);
    const clean = {};
    for (const [name, value] of Object.entries(values)) {
      clean[name] = finite(value) ? value : null;
      const stats = this.fields[`${kind}.${name}`] ??= { valid: 0, invalid: 0, min: null, max: null };
      if (clean[name] === null) stats.invalid++;
      else {
        stats.valid++; stats.min = stats.min === null ? value : Math.min(stats.min, value);
        stats.max = stats.max === null ? value : Math.max(stats.max, value);
      }
    }
    this[kind] = { ...clean, at: now };
    this.samples.push({ kind, atMs: now, ...clean });
    if (this.samples.length > 120) this.samples.shift();
    if (kind === 'orientation' && ['alpha', 'beta', 'gamma'].some(key => clean[key] === null)) {
      this.invalidate('Missing orientation axis; recalibration required', now);
    }
    if (kind === 'orientation' && this.neutral && this.stationary && !this.stationary.saved) {
      const delta = relative(clean, this.neutral), trial = this.stationary;
      trial.lastAtMs = now; trial.count++; trial.lastDeltaDegrees = delta;
      for (const key of ['alpha', 'beta', 'gamma']) {
        trial.minDeltaDegrees[key] = Math.min(trial.minDeltaDegrees[key], delta[key]);
        trial.maxDeltaDegrees[key] = Math.max(trial.maxDeltaDegrees[key], delta[key]);
      }
    }
  }
  fresh(now) {
    const value = this.orientation;
    return this.enabled && value && now >= value.at && now - value.at < STALE_MS &&
      ['alpha', 'beta', 'gamma'].every(key => finite(value[key]));
  }
  calibrate(now) {
    if (!this.fresh(now)) return false;
    this.neutral = { ...this.orientation }; this.held = false; this.holdAt = null;
    this.stationary = { startedAtMs: now, lastAtMs: now, count: 0, saved: false,
      minDeltaDegrees: { alpha: 0, beta: 0, gamma: 0 }, maxDeltaDegrees: { alpha: 0, beta: 0, gamma: 0 },
      lastDeltaDegrees: { alpha: 0, beta: 0, gamma: 0 } };
    this.reason = 'Calibrated; hold preview to test'; this.log('calibrated', now); return true;
  }
  press(now) {
    if (!this.neutral || !this.fresh(now)) return false;
    this.held = true; this.holdAt = now; return true;
  }
  release() { this.held = false; this.holdAt = null; }
  saveStationary(now) {
    if (!this.stationary || this.stationary.saved || !this.fresh(now)) return false;
    this.stationary.saved = true;
    this.stationaryResults.push(structuredClone(this.stationary));
    if (this.stationaryResults.length > 6) this.stationaryResults.shift();
    this.log('Stationary result saved; human stillness unverified', now); return true;
  }
  frame(now) {
    this.cadences.ui.record(now);
    if (this.held) {
      if (now - this.holdAt >= STALE_MS) this.invalidate('UI renewal expired; recalibrate', now);
      else this.holdAt = now;
    }
  }
  consume(now) {
    if (this.enabled) this.cadences.consumer.record(now);
    if (this.neutral && !this.fresh(now)) this.invalidate('Orientation expired; recalibrate', now);
    if (this.held && now - this.holdAt >= STALE_MS) this.invalidate('UI renewal expired; recalibrate', now);
    const zero = { forward: 0, left: 0, rotateLeft: 0 };
    if (!this.neutral || !this.held || !this.fresh(now)) return zero;
    return { forward: scaleAngle(-angleDelta(this.orientation.beta, this.neutral.beta)),
      left: scaleAngle(-angleDelta(this.orientation.gamma, this.neutral.gamma)),
      rotateLeft: scaleAngle(angleDelta(this.orientation.alpha, this.neutral.alpha)) };
  }
  report(now) {
    return { diagnosticOnly: true, staleCutoffMs: STALE_MS, previewDeadZoneDegrees: 3,
      previewFullScaleDegrees: 20, angleMapping: 'negative beta -> forward; negative gamma -> left; positive alpha -> rotate left',
      timestamps: 'performance.now arrival times, not sensor acquisition times',
      statistics: 'counts/max cover enabled segments including interruptions; median/p95 use latest 4096 gaps',
      reason: this.reason, neutral: this.neutral, stationaryResults: this.stationaryResults,
      cadences: Object.fromEntries(Object.entries(this.cadences).map(([key, value]) => [key, value.summary(now)])),
      fields: this.fields, lastOrientation: this.orientation, lastMotion: this.motion,
      events: this.events, recentSamples: this.samples };
  }
}
