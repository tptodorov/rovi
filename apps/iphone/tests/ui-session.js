import { Hold } from './hold.js';
import { Physical } from './physical.js';
import { ZERO } from './wire.js';
import { mount } from './ui.js';

const session = {
  phase: 'ready', localArm: false, control: null, message: 'Fake session: UI test only',
  transport: { connected: true }, device: { name: 'Fake car', id: 'test-only' },
  caps: { max_vx: 0.05, max_vy: 0.1, max_wz: 0.2, lease_ms: 1000, claim_ms: 10000, reclaim_ms: 5000 },
  hold: new Hold(),
  physical: new Physical(), mode: 'buttons',
  clearInputs() { this.hold.clear(); this.physical.clear(); },
  get requested() { return this.localArm ? this.hold.intent(this.caps, performance.now()) : ZERO; },
  arm() { this.hold.clear(); this.localArm = true; this.changed(); },
  stop() { this.hold.clear(); this.localArm = false; this.changed(); },
  close() { this.stop(); this.phase = 'disconnected'; this.transport.connected = false; this.changed(); },
  inactive() { this.close(); },
  press(id, key) { this.hold.press(id, key, this.localArm, performance.now()); },
  release(id) { this.hold.release(id); },
};
window.fixture = session;
mount(session);
