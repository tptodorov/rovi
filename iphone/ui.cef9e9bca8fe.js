import { DIRECTIONS, ZERO } from './wire.cef9e9bca8fe.js';

export function mount(session) {
  const byId = id => document.getElementById(id);
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
  byId('control-mode').addEventListener('change', event => {
    detachSensors(); session.selectMode(event.target.value);
  });
  byId('sensors-enable').addEventListener('click', async () => {
    if (session.mode !== 'physical' || !active() || session.localArm || session.control || !['disconnected', 'ready'].includes(session.phase)) return;
    detachSensors(); session.physical.disable('Requesting sensor permission…');
    const epoch = sensorEpoch, generation = session.generation;
    permissionPending = true;
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
  });
  byId('connect').addEventListener('click', () => { void session.choose(); });
  byId('disconnect').addEventListener('click', () => { void session.close('Disconnect requested'); });
  byId('arm').addEventListener('click', () => { void session.arm(); });
  // iOS sends no click for a second finger, so STOP acts on touch-down; its later click is ignored.
  let stopTouchedAt = -Infinity;
  byId('stop').addEventListener('pointerdown', event => {
    if (event.pointerType === 'mouse' && event.button !== 0) return;
    stopTouchedAt = performance.now(); void session.stop();
  });
  byId('stop').addEventListener('click', () => { if (performance.now() - stopTouchedAt > 1000) void session.stop(); });
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
  function render() {
    const physical = session.physical, physicalMode = session.mode === 'physical';
    if (!physical.enabled && sensorListener) detachSensors();
    const physicalReady = physical.ready(performance.now());
    byId('control-mode').value = session.mode;
    byId('button-controls').hidden = physicalMode;
    byId('physical-controls').hidden = !physicalMode;
    byId('hold-hint').textContent = physicalMode ? 'Set neutral & ARM · tap START · tilt to move · tap PAUSE for zero' : 'Hold to move · release to zero · STOP requires a new ARM';
    byId('sensors-enable').disabled = permissionPending || session.localArm || Boolean(session.control) || !['disconnected', 'ready'].includes(session.phase);
    byId('physical-status').textContent = physical.problem || (physicalReady ? (physical.driving ? 'Tilt control running. Tap PAUSE for zero or STOP to disarm.' : 'Tilt control paused. Return to neutral, then tap START after ARM.') : 'Drive paused: wait for fresh readings and return to the screen-up operating posture.');
    const sample = physical.sample;
    byId('physical-reading').textContent = sample ? `α ${sample.alpha.toFixed(1)}° · β ${sample.beta.toFixed(1)}° · γ ${sample.gamma.toFixed(1)}° · age ${Math.max(0, performance.now() - sample.at).toFixed(0)} ms` : 'No sensor reading';
    drive.hidden = !physicalMode;
    byId('physical-requested').hidden = !physicalMode;
    drive.disabled = !physicalMode || session.phase !== 'ready' || !session.localArm || Boolean(session.control) || (!physical.driving && !physical.canStart(performance.now())) || !['vx', 'vy', 'wz'].some(axis => session.caps?.[`max_${axis}`] > 0);
    drive.textContent = physical.driving ? 'PAUSE TILT CONTROL' : 'START TILT CONTROL';
    drive.setAttribute('aria-pressed', String(physical.driving));
    byId('phase').textContent = session.phase;
    document.body.dataset.phase = session.phase;
    document.body.dataset.armed = String(session.localArm);
    byId('connect').disabled = session.phase !== 'disconnected';
    byId('disconnect').disabled = session.phase === 'disconnected' || session.phase === 'closing';
    byId('arm').textContent = physicalMode ? 'SET NEUTRAL & ARM' : 'ARM AT ZERO';
    byId('arm').disabled = session.phase !== 'ready' || session.localArm || Boolean(session.control) || (physicalMode && !physical.canCalibrate(performance.now()));
    byId('stop').disabled = !session.transport?.connected || session.phase === 'closing';
    byId('local-arm').textContent = session.localArm ? 'Locally armed' : 'Locally disarmed';
    byId('message').textContent = session.message;
    byId('device').textContent = session.device ? `${session.device.name || 'Selected car'}${session.device.id ? ` · ${session.device.id}` : ''}` : 'No car selected';
    const motion = session.requested ?? ZERO;
    byId('requested').textContent = `vx ${motion.vx.toFixed(2)} · vy ${motion.vy.toFixed(2)} · wz ${motion.wz.toFixed(2)}`;
    byId('physical-requested').textContent = `Requested: ${byId('requested').textContent}`;
    if (session.reported) {
      const { state, reason, error } = session.reported;
      byId('reported').textContent = `${state} · reason ${reason} · error ${error}`;
      byId('status-age').textContent = `${Math.max(0, (performance.now() - session.reportedAt) / 1000).toFixed(1)} s ago`;
    }
    if (session.caps) {
      const caps = session.caps;
      byId('limits').textContent = `Limits: vx ${caps.max_vx} m/s · vy ${caps.max_vy} m/s · wz ${caps.max_wz} rad/s`;
      byId('windows').textContent = `Lease ${caps.lease_ms} ms · claim ${caps.claim_ms} ms · reclaim ${caps.reclaim_ms} ms`;
    }
    const { writes, maxGap, maxWrite } = session.stats ?? {};
    if (writes) byId('stream').textContent = `Stream: ${writes} writes · max gap ${maxGap.toFixed(0)} ms · slowest write ${maxWrite.toFixed(0)} ms`;
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
