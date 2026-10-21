import { DIRECTIONS, ZERO } from './wire.2b5104400362.js';
import { angleDelta } from './angles.2b5104400362.js';

export function mount(session) {
  const byId = id => document.getElementById(id);
  const text = (id, value) => { const node = byId(id); if (node.textContent !== value) node.textContent = value; };
  const modes = [...document.querySelectorAll('[name="control-mode"]')];
  const buttons = [...document.querySelectorAll('[data-direction]')];
  const drive = byId('physical-drive');
  const pointers = new Map();
  const active = () => document.visibilityState === 'visible' && document.hasFocus();
  session.active = active;
  let sensorEpoch = 0, sensorListener = null, permissionPending = false;
  function detachSensors() {
    sensorEpoch++; permissionPending = false;
    if (sensorListener) window.removeEventListener('deviceorientation', sensorListener);
    sensorListener = null;
  }
  for (const mode of modes) mode.addEventListener('change', event => {
    if (!event.target.checked) return;
    detachSensors(); session.selectMode(event.target.value);
    if (event.target.value === 'physical') void enableSensors();
  });
  async function enableSensors() {
    if (session.mode !== 'physical' || !active() || session.localArm || session.control?.kind === 'arm' || !['disconnected', 'ready'].includes(session.phase)) return;
    detachSensors(); session.physical.disable('Requesting sensor permission…');
    const epoch = sensorEpoch, generation = session.generation;
    permissionPending = true; render();
    try {
      const API = window.DeviceOrientationEvent;
      if (!API) throw new Error('Orientation sensors are unavailable in this browser.');
      const result = typeof API.requestPermission === 'function' ? await API.requestPermission() : 'granted';
      if (epoch !== sensorEpoch || generation !== session.generation || session.mode !== 'physical' || !active()) return;
      if (result !== 'granted') throw new Error('Sensor permission denied; enable it before using Physical mode.');
      session.physical.enable();
      sensorListener = event => {
        if (epoch === sensorEpoch && session.mode === 'physical' && session.physical.enabled && active()) session.sensor(event);
      };
      window.addEventListener('deviceorientation', sensorListener);
    } catch (error) {
      if (epoch === sensorEpoch) session.physical.disable(error.message);
    } finally {
      if (epoch === sensorEpoch) permissionPending = false;
      render();
    }
  }
  byId('sensors-enable').addEventListener('click', () => { void enableSensors(); });
  byId('connect').addEventListener('click', () => { void session.choose(); });
  byId('disconnect').addEventListener('click', () => { void session.close('Disconnect requested'); });
  byId('arm').addEventListener('click', () => { void session.arm(); });
  function stop() {
    if (permissionPending) { detachSensors(); session.physical.disable('Motion setup cancelled. Retry motion access when ready.'); }
    void session.stop();
  }
  // iOS sends no click for a second finger, so STOP acts on touch-down; its later click is ignored.
  let stopTouchedAt = -Infinity;
  byId('stop').addEventListener('pointerdown', event => {
    if (event.pointerType === 'mouse' && event.button !== 0) return;
    stopTouchedAt = performance.now(); stop();
  });
  byId('stop').addEventListener('click', () => { if (performance.now() - stopTouchedAt > 1000) stop(); });
  drive.addEventListener('click', () => session.togglePhysical());
  for (const button of buttons) {
    button.addEventListener('contextmenu', event => event.preventDefault());
    button.addEventListener('click', event => event.preventDefault()); // Click/accessibility activation cannot latch motion.
    button.addEventListener('pointerdown', event => {
      if (button.disabled || (event.pointerType === 'mouse' && event.button !== 0)) return;
      event.preventDefault();
      pointers.set(event.pointerId, button);
      button.setPointerCapture(event.pointerId);
      session.press(event.pointerId, button.dataset.direction);
    });
    button.addEventListener('pointermove', event => {
      if (!pointers.has(event.pointerId)) return;
      const bounds = button.getBoundingClientRect();
      if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) {
        session.hold.clear(); session.physical.clear();
      }
    });
    const release = event => {
      pointers.delete(event.pointerId);
      session.release(event.pointerId);
    };
    button.addEventListener('pointerup', release);
    button.addEventListener('pointercancel', release);
    button.addEventListener('lostpointercapture', release);
  }
  document.addEventListener('scroll', () => {
    if (session.physical.driving) session.message = 'Scrolling paused tilt control at zero. Return to neutral, then tap START.';
    session.clearInputs(); render();
  }, { capture: true, passive: true });
  const inactive = () => { detachSensors(); session.physical.disable(); pointers.clear(); void session.inactive(); };
  window.addEventListener('blur', () => { if (session.phase !== 'choosing') inactive(); });
  document.addEventListener('visibilitychange', () => { if (document.hidden) inactive(); });
  window.addEventListener('pagehide', inactive);
  const screenChanged = () => { if (session.mode === 'physical') inactive(); };
  window.addEventListener('orientationchange', screenChanged);
  screen.orientation?.addEventListener?.('change', screenChanged);
  byId('diagnostics').addEventListener('toggle', () => {
    if (!byId('diagnostics').open) return;
    if (session.physical.driving) session.message = 'Diagnostics opened; tilt control paused. Return to neutral, then tap START.';
    session.clearInputs(); render();
  });
  const theme = window.matchMedia('(prefers-color-scheme: dark)');
  const themeColor = () => { document.querySelector('meta[name="theme-color"]').content = theme.matches ? '#0f1a15' : '#f3f5ef'; };
  theme.addEventListener('change', themeColor); themeColor();
  function render() {
    const now = performance.now(), physical = session.physical, physicalMode = session.mode === 'physical';
    if (!physical.enabled && sensorListener) detachSensors();
    const physicalReady = physical.ready(now), canStart = physical.canStart(now);
    const ready = session.phase === 'ready';
    for (const mode of modes) mode.checked = mode.value === session.mode;
    byId('button-controls').hidden = physicalMode;
    byId('physical-controls').hidden = !physicalMode;
    text('surface-title', physicalMode ? 'Phone input' : 'Your move.');
    text('input-state', physicalMode ? (physical.driving ? 'LIVE INPUT' : 'PAUSED') : 'HOLD TO DRIVE');
    text('hold-hint', physicalMode ? 'Tap START at neutral. Tilt to move; PAUSE for zero.' : 'Hold a direction to move. Release for zero.');
    byId('sensors-enable').hidden = !physicalMode || permissionPending || Boolean(physical.fresh(now));
    byId('sensors-enable').disabled = session.localArm || Boolean(session.control) || !['disconnected', 'ready'].includes(session.phase);
    const sensorStatus = permissionPending ? 'Allow motion access in the browser prompt…' : physical.problem || (physicalReady ?
      (physical.driving ? 'Tilt control is live until PAUSE or STOP.' : canStart ? 'At neutral. Ready for START after ARM.' : 'Return to calibrated neutral before START.') :
      'Waiting for fresh readings. Keep the phone screen up.');
    text('physical-status', sensorStatus);
    text('tilt-neutral', !physical.neutral ? 'Set your neutral position' : canStart ? 'At neutral' : physicalReady ? 'Phone input' : 'Input unavailable');
    const sample = physical.sample;
    text('physical-reading', sample ? `α ${sample.alpha.toFixed(1)}° · β ${sample.beta.toFixed(1)}° · γ ${sample.gamma.toFixed(1)}° · age ${Math.max(0, now - sample.at).toFixed(0)} ms` : 'No sensor reading');
    const offsets = sample && physical.neutral ? {
      alpha: angleDelta(sample.alpha, physical.neutral.alpha), beta: angleDelta(sample.beta, physical.neutral.beta), gamma: angleDelta(sample.gamma, physical.neutral.gamma),
    } : { alpha: 0, beta: 0, gamma: 0 };
    const limit = (value, maximum) => Math.max(-maximum, Math.min(maximum, value));
    byId('phone-pose').setAttribute('transform', `translate(${150 + limit(offsets.gamma, 20) * 1.5} ${75 + limit(offsets.beta, 20)}) rotate(${-limit(offsets.alpha, 60)})`);
    drive.hidden = !physicalMode || !session.localArm;
    drive.disabled = !physicalMode || !ready || !session.localArm || Boolean(session.control) || (!physical.driving && !canStart) || !['vx', 'vy', 'wz'].some(axis => session.caps?.[`max_${axis}`] > 0);
    text('physical-drive', physical.driving ? 'PAUSE TILT CONTROL' : 'START TILT CONTROL');
    drive.setAttribute('aria-pressed', String(physical.driving));
    const phases = { disconnected: 'Not connected', choosing: 'Choose your Rovi', connecting: 'Connecting…', ready: 'Connected', closing: 'Disconnecting…' };
    text('phase', phases[session.phase] ?? session.phase);
    document.body.dataset.phase = session.phase;
    document.body.dataset.armed = String(session.localArm);
    document.body.dataset.driving = String(session.localArm && (physicalMode ? physical.driving : session.hold.current !== null));
    document.body.dataset.mode = session.mode;
    byId('connect').hidden = session.phase !== 'disconnected';
    byId('connect').disabled = session.phase !== 'disconnected';
    byId('disconnect').hidden = session.phase === 'disconnected';
    byId('disconnect').disabled = session.phase === 'disconnected' || session.phase === 'closing';
    byId('arm').hidden = physicalMode && session.localArm;
    text('arm', physicalMode ? 'SET NEUTRAL & ARM' : 'ARM AT ZERO');
    byId('arm').disabled = !ready || session.localArm || Boolean(session.control) || (physicalMode && !physical.canCalibrate(now));
    byId('stop').disabled = !session.transport?.connected || session.phase === 'closing';
    text('local-arm', session.localArm ? 'Armed' : 'Disarmed');
    text('stage-title', !ready ? (session.phase === 'disconnected' ? (session.device ? 'Connection ended' : 'Connect your Rovi') : phases[session.phase]) :
      !session.localArm ? (physicalMode && !physical.canCalibrate(now) ? (permissionPending ? 'Allow phone motion' : physical.fresh(now) ? 'Hold your phone flat' : 'Waiting for sensors') : 'Ready to arm') :
      !physicalMode ? 'Hold to drive' : physical.driving ? 'Tilt control live' : canStart ? 'Ready to tilt' : 'Return to neutral');
    text('message', session.message);
    text('device', session.device ? `${session.device.name || 'Selected car'}${session.device.id ? ` · ${session.device.id}` : ''}` : 'No car selected');
    const motion = session.requested ?? ZERO;
    text('requested', `vx ${motion.vx.toFixed(2)} · vy ${motion.vy.toFixed(2)} · wz ${motion.wz.toFixed(2)}`);
    for (const axis of ['vx', 'vy', 'wz']) text(`value-${axis}`, motion[axis].toFixed(2));
    if (session.reported) {
      const { state, reason, error } = session.reported;
      text('reported', `${state} · reason ${reason} · error ${error}`);
      text('status-age', `${Math.max(0, (now - session.reportedAt) / 1000).toFixed(1)} s ago`);
    }
    if (session.caps) {
      const caps = session.caps;
      text('limits', `Limits: vx ${caps.max_vx} m/s · vy ${caps.max_vy} m/s · wz ${caps.max_wz} rad/s`);
      text('windows', `Lease ${caps.lease_ms} ms · claim ${caps.claim_ms} ms · reclaim ${caps.reclaim_ms} ms`);
    }
    const { writes, maxGap, maxWrite } = session.stats ?? {};
    if (writes) text('stream', `Stream: ${writes} writes · max gap ${maxGap.toFixed(0)} ms · slowest write ${maxWrite.toFixed(0)} ms`);
    for (const button of buttons) {
      const [axis] = DIRECTIONS[button.dataset.direction];
      button.disabled = physicalMode || !session.localArm || !session.caps?.[`max_${axis}`];
      button.dataset.held = String(session.hold.current !== null && session.hold.pointers.get(session.hold.current) === button.dataset.direction);
    }
  }
  session.changed = render;
  function frame() {
    if (active()) {
      session.hold.renew(performance.now());
      if (session.mode === 'physical') session.physical.renew(performance.now());
    }
    else if (session.localArm) inactive();
    render();
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
  render();
}
