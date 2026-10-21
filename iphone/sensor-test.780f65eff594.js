import { SensorProbe } from './sensor-model.780f65eff594.js';

const probe = new SensorProbe(), byId = id => document.getElementById(id), now = () => performance.now();
const active = () => !document.hidden && document.hasFocus();
let generation = 0, sensorsPaused = false, uiPaused = false, pointer = null, absolute = null;
let permissions = {}, revision = 'Source checkout', preview = { forward: 0, left: 0, rotateLeft: 0 };
const stallTimers = { sensors: null, ui: null };
const number = value => Number.isFinite(value) ? value.toFixed(1) : 'missing';
const release = () => { pointer = null; probe.release(); };
const orientation = event => {
  if (!active() || sensorsPaused || !probe.enabled) return;
  if (absolute !== null && absolute !== event.absolute) probe.invalidate('Orientation reference changed; recalibrate', now());
  absolute = event.absolute;
  probe.record('orientation', { alpha: event.alpha, beta: event.beta, gamma: event.gamma,
    compassHeading: event.webkitCompassHeading, compassAccuracy: event.webkitCompassAccuracy,
    eventTimeStamp: event.timeStamp }, now());
};
const motion = event => {
  if (!active() || sensorsPaused || !probe.enabled) return;
  probe.record('motion', { ax: event.acceleration?.x, ay: event.acceleration?.y, az: event.acceleration?.z,
    gx: event.accelerationIncludingGravity?.x, gy: event.accelerationIncludingGravity?.y, gz: event.accelerationIncludingGravity?.z,
    rotationAlpha: event.rotationRate?.alpha, rotationBeta: event.rotationRate?.beta, rotationGamma: event.rotationRate?.gamma,
    interval: event.interval, eventTimeStamp: event.timeStamp }, now());
};
function stop(reason) {
  generation++; release(); probe.disable(reason, now());
  window.removeEventListener('deviceorientation', orientation); window.removeEventListener('devicemotion', motion);
  for (const field of ['sensors', 'ui']) { clearTimeout(stallTimers[field]); stallTimers[field] = null; }
  sensorsPaused = false; uiPaused = false; preview = { forward: 0, left: 0, rotateLeft: 0 }; render();
}
function report() {
  return JSON.stringify({ revision, origin: location.origin, userAgent: navigator.userAgent, secureContext: isSecureContext,
    versions: byId('versions').value, observations: byId('observations').value, permissions, orientationAbsolute: absolute,
    ...probe.report(now()) }, null, 2);
}
byId('enable').addEventListener('click', async () => {
  stop('Requesting sensor permissions'); const current = generation;
  const APIs = { orientation: window.DeviceOrientationEvent, motion: window.DeviceMotionEvent };
  // Both permission calls must run within this tap, before any await.
  const requests = Object.entries(APIs).map(([name, API]) => {
    try {
      return Promise.resolve(!API ? 'API missing' : typeof API.requestPermission === 'function' ? API.requestPermission() : 'no permission method')
        .then(result => [name, result], error => [name, `error: ${error.message}`]);
    } catch (error) { return Promise.resolve([name, `error: ${error.message}`]); }
  });
  const results = Object.fromEntries(await Promise.all(requests));
  if (current !== generation) return;
  permissions = results;
  byId('permissions').textContent = Object.entries(permissions).map(([name, result]) => `${name}: ${result}`).join(' · ');
  if (!active()) return;
  probe.enable(now()); absolute = null;
  const allowed = result => ['granted', 'no permission method'].includes(result);
  if (allowed(permissions.orientation)) window.addEventListener('deviceorientation', orientation);
  if (allowed(permissions.motion)) window.addEventListener('devicemotion', motion);
  render();
});
byId('disable').addEventListener('click', () => { stop('Finished by user'); byId('report').value = report(); });
byId('calibrate').addEventListener('click', () => { release(); probe.calibrate(now()); render(); });
byId('stationary').addEventListener('click', () => { probe.saveStationary(now()); render(); });
byId('hold').addEventListener('pointerdown', event => {
  event.preventDefault();
  if (pointer !== null || !event.isPrimary || (event.pointerType === 'mouse' && event.button !== 0)) { release(); return; }
  if (active() && probe.press(now())) { pointer = event.pointerId; byId('hold').setPointerCapture(pointer); }
});
byId('hold').addEventListener('pointermove', event => {
  const box = byId('hold').getBoundingClientRect();
  if (event.pointerId === pointer && (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom)) release();
});
for (const name of ['pointerup', 'pointercancel', 'lostpointercapture']) window.addEventListener(name, release);
document.addEventListener('pointerdown', event => { if (pointer !== null && event.pointerId !== pointer) release(); }, true);
for (const [id, field] of [['freeze-sensors', 'sensors'], ['freeze-ui', 'ui']]) byId(id).addEventListener('click', () => {
  if (field === 'sensors' ? sensorsPaused : uiPaused) {
    if (field === 'sensors') sensorsPaused = false; else uiPaused = false;
    release(); probe.invalidate(`${field} resumed; recalibrate`, now());
  } else {
    const current = generation;
    stallTimers[field] = setTimeout(() => {
      stallTimers[field] = null;
      if (current !== generation || !probe.enabled) return;
      if (field === 'sensors') sensorsPaused = true; else uiPaused = true;
      probe.log(`${field} stall started`, now());
    }, 2000);
  }
  render();
});
byId('copy').addEventListener('click', async () => {
  const text = report(); byId('report').value = text;
  try { await navigator.clipboard.writeText(text); byId('copy').textContent = 'Copied'; }
  catch { byId('report').focus(); byId('report').select(); byId('copy').textContent = 'Select report and copy'; }
});
for (const name of ['blur', 'pagehide', 'orientationchange']) window.addEventListener(name, () => stop(`${name}; enable and calibrate again`));
screen.orientation?.addEventListener?.('change', () => stop('Screen orientation changed; enable and calibrate again'));
document.addEventListener('visibilitychange', () => { if (document.hidden) stop('Page hidden; enable and calibrate again'); });

function render() {
  const at = now(), o = probe.orientation, m = probe.motion;
  byId('message').textContent = probe.reason;
  byId('orientation').textContent = o ? `Orientation °: α ${number(o.alpha)} · β ${number(o.beta)} · γ ${number(o.gamma)} · age ${number(at - o.at)} ms` : 'Orientation: no readings';
  byId('motion').textContent = m ? `Gravity m/s²: ${number(m.gx)}, ${number(m.gy)}, ${number(m.gz)}. Rotation °/s: α ${number(m.rotationAlpha)} · β ${number(m.rotationBeta)} · γ ${number(m.rotationGamma)}` : 'Motion: no readings';
  byId('cadence').textContent = ['orientation', 'motion'].map(key => {
    const stats = probe.cadences[key].summary(at);
    return `${key}: ${stats.count} events, age ${number(stats.ageMs)} ms, median / p95 / max gap ${number(stats.medianGapMs)} / ${number(stats.p95GapMs)} / ${number(stats.maxGapMs)} ms`;
  }).join(' | ');
  byId('preview').textContent = `Forward ${(preview.forward * 100).toFixed(0)}% · Left ${(preview.left * 100).toFixed(0)}% · Rotate left ${(preview.rotateLeft * 100).toFixed(0)}%`;
  byId('calibrate').disabled = !probe.fresh(at) || sensorsPaused || uiPaused;
  byId('stationary').disabled = !probe.fresh(at) || !probe.stationary || probe.stationary.saved;
  byId('stationary').textContent = probe.stationary?.saved ? 'Stationary result saved' : 'Save stationary result';
  byId('stationary-age').textContent = probe.stationary ? `Neutral trial: ${Math.floor((probe.stationary.lastAtMs - probe.stationary.startedAtMs) / 1000)} s · target 60 s${probe.stationary.saved ? ' · saved' : ''}` : 'Neutral trial: not calibrated';
  byId('hold').disabled = !probe.neutral || !probe.enabled; byId('hold').dataset.held = String(probe.held);
  byId('disable').disabled = !probe.enabled;
  byId('freeze-sensors').disabled = !probe.enabled || stallTimers.sensors !== null;
  byId('freeze-ui').disabled = !probe.enabled || stallTimers.ui !== null;
  byId('freeze-sensors').textContent = stallTimers.sensors !== null ? 'Sensor pause scheduled…' : sensorsPaused ? 'Resume sensor input' : 'Pause sensor input in 2s';
  byId('freeze-ui').textContent = stallTimers.ui !== null ? 'Hold pause scheduled…' : uiPaused ? 'Resume hold renewal' : 'Pause hold renewal in 2s';
}
function frame() { if (active() && probe.enabled && !uiPaused) probe.frame(now()); requestAnimationFrame(frame); }
requestAnimationFrame(frame);
setInterval(() => {
  if (probe.enabled && !active()) stop('Inactive; enable and calibrate again');
  preview = probe.consume(now()); render();
}, 50);
const build = document.querySelector('meta[name="rovi-build"]')?.content;
if (build && build !== 'source-checkout') { revision = build; byId('version').textContent = build; }
render();
