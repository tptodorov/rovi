import { DIRECTIONS, ZERO } from './wire.js';

export function mount(session) {
  const byId = id => document.getElementById(id);
  const buttons = [...document.querySelectorAll('[data-direction]')];
  const pointers = new Map();
  const active = () => document.visibilityState === 'visible' && document.hasFocus();
  session.active = active;
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
      if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) session.hold.clear();
    });
    const release = event => { pointers.delete(event.pointerId); session.release(event.pointerId); };
    button.addEventListener('pointerup', release);
    button.addEventListener('pointercancel', release);
    button.addEventListener('lostpointercapture', release);
  }
  const inactive = () => { pointers.clear(); void session.inactive(); };
  window.addEventListener('blur', () => { if (session.phase !== 'choosing') inactive(); });
  document.addEventListener('visibilitychange', () => { if (document.hidden) inactive(); });
  window.addEventListener('pagehide', inactive);
  function render() {
    byId('phase').textContent = session.phase;
    document.body.dataset.phase = session.phase;
    byId('connect').disabled = session.phase !== 'disconnected';
    byId('disconnect').disabled = session.phase === 'disconnected' || session.phase === 'closing';
    byId('arm').disabled = session.phase !== 'ready' || session.localArm || Boolean(session.control);
    byId('stop').disabled = !session.transport?.connected || session.phase === 'closing';
    byId('local-arm').textContent = session.localArm ? 'Locally armed' : 'Locally disarmed';
    byId('message').textContent = session.message;
    byId('device').textContent = session.device ? `${session.device.name || 'Selected car'}${session.device.id ? ` · ${session.device.id}` : ''}` : 'No car selected';
    const motion = session.requested ?? ZERO;
    byId('requested').textContent = `vx ${motion.vx.toFixed(2)} · vy ${motion.vy.toFixed(2)} · wz ${motion.wz.toFixed(2)}`;
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
      button.disabled = !session.localArm || !session.caps?.[`max_${axis}`];
      button.dataset.held = String(session.hold.current !== null && session.hold.pointers.get(session.hold.current) === button.dataset.direction);
    }
  }
  session.changed = render;
  function frame() {
    if (active()) session.hold.renew(performance.now());
    else if (session.localArm) inactive();
    render();
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
  render();
}
