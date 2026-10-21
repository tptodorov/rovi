import { capabilities, status, setpoint, ZERO, DIRECTIONS } from './wire.780f65eff594.js';
import { Hold } from './hold.780f65eff594.js';
import { Physical } from './physical.780f65eff594.js';
import { Ble, pickDevice, supportError } from './transport.780f65eff594.js';

export class Session {
  phase = 'disconnected';
  localArm = false;
  hold = new Hold();
  physical = new Physical();
  mode = 'buttons';
  control = null;
  transport = null;
  device = null;
  caps = null;
  reported = null;
  reportedAt = 0;
  generation = 0;
  seq = 0;
  stats = { writes: 0, maxGap: 0, maxWrite: 0 };
  busy = null;
  cleanup = null;
  changed = () => {};

  constructor({ bluetooth, protocol, now = () => performance.now(), active = () => true,
    makeTransport = (device, onStatus, onLost) => new Ble(device, onStatus, onLost) } = {}) {
    Object.assign(this, { bluetooth, protocol, now, active, makeTransport });
    this.message = supportError(bluetooth, protocol) || 'Choose a car. Connecting claims it; ARM is always explicit.';
  }

  get requested() {
    if (!this.localArm || this.phase !== 'ready' || !this.active()) return ZERO;
    return (this.mode === 'physical' ? this.physical : this.hold).intent(this.caps, this.now());
  }

  clearInputs() { this.hold.clear(); this.physical.clear(); }
  selectMode(mode) {
    if (!['buttons', 'physical'].includes(mode) || mode === this.mode) return;
    this.mode = mode; this.localArm = false; this.clearInputs(); this.physical.disable();
    if (this.phase === 'ready') this.stop();
    else if (['choosing', 'connecting'].includes(this.phase)) void this.close('Control mode changed; connect again.');
    this.changed();
  }
  sensor(value) {
    if (this.mode !== 'physical' || !this.active()) return;
    this.physical.update(value, this.now()); this.checkPhysical(); this.changed();
  }
  pressPhysical(id) {
    this.physical.press(id, this.mode === 'physical' && this.localArm && this.phase === 'ready' && this.active(), this.now());
    this.changed();
  }
  checkPhysical() {
    if (this.mode === 'physical' && (this.localArm || this.control?.kind === 'arm') && !this.physical.ready(this.now()) &&
        (this.physical.problem || this.control?.kind === 'arm')) {
      void this.close(this.physical.problem || 'Physical input unavailable; reconnect and calibrate.'); return false;
    }
    return true;
  }

  press(id, name) {
    const axis = DIRECTIONS[name]?.[0];
    this.hold.press(id, name, this.mode === 'buttons' && this.localArm && this.active() && this.caps?.[`max_${axis}`] > 0, this.now());
    this.changed();
  }
  release(id) { this.hold.release(id); this.changed(); }

  async choose() {
    if (this.phase !== 'disconnected' || !this.active()) return;
    const unsupported = supportError(this.bluetooth, this.protocol);
    if (unsupported) { this.message = unsupported; this.changed(); return; }
    const generation = ++this.generation;
    this.phase = 'choosing'; this.localArm = false; this.hold.reset();
    this.physical.hold.reset(); this.physical.invalidate('Hold neutral, then tap Set neutral & ARM.');
    this.caps = null; this.reported = null; this.device = null; this.seq = 0;
    this.stats = { writes: 0, maxGap: 0, maxWrite: 0 };
    this.message = 'Select the car in Bluefy’s picker. The picker alone does not claim it.'; this.changed();
    try {
      const device = await pickDevice(this.bluetooth, this.protocol);
      if (generation !== this.generation) return;
      if (!device || !this.active()) throw new Error('Selection cancelled or page inactive; choose again');
      this.device = device;
      this.phase = 'connecting'; this.setupDeadline = this.now() + 10000;
      this.message = 'Connecting and validating; no ARM or motion is sent.';
      this.transport = this.makeTransport(device,
        bytes => { if (generation === this.generation) this.notification(bytes); },
        () => { if (generation === this.generation) void this.close('Bluetooth connection lost'); });
      const transport = this.transport;
      this.job('setup', async () => {
        const initial = await transport.open();
        if (generation !== this.generation || !this.checkDeadlines()) return;
        this.caps = capabilities(initial.caps);
        const report = status(initial.report);
        if (!['Claimed', 'Stopped'].includes(report.state)) throw new Error('Car did not report a disarmed claim');
        this.observe(initial.report);
        if (generation !== this.generation || !this.checkDeadlines()) return;
        await this.sendZero(transport);
      }, () => {
        this.phase = 'ready'; this.lastTick = this.now(); this.lastRead = this.now();
        this.message = 'Connected and locally disarmed. Choose ARM to prepare at zero.';
      });
    } catch (error) {
      if (generation === this.generation) void this.close(`Connection failed: ${error.message}`);
    }
    this.changed();
  }

  notification(bytes) {
    if (!['connecting', 'ready'].includes(this.phase)) return;
    try { this.observe(bytes); } catch (error) { void this.close(error.message); }
  }

  observe(bytes) {
    const next = status(bytes), previous = this.reported;
    this.reported = next; this.reportedAt = this.now();
    if (next.state === 'Open') throw new Error('Car reported ownership open; connect again');
    if (next.state === 'Armed' && !this.localArm && !this.control) throw new Error('Unexpected Armed Status; motion permission revoked');
    if (next.state === 'Stopped') {
      this.localArm = false; this.clearInputs();
      if (this.control?.kind === 'arm' && (previous?.state !== 'Stopped' || previous.reason !== next.reason)) {
        throw new Error(`Safety stop during ARM: ${next.reason}`);
      }
      this.message = `Car reported Stopped/${next.reason}. A fresh ARM and new press are required.`;
    }
    this.changed();
    return next;
  }

  arm() {
    if (this.phase !== 'ready' || this.localArm || this.control || !this.active()) return;
    if (this.mode === 'physical' && !this.physical.calibrate(this.now())) {
      this.message = this.physical.problem; this.changed(); return;
    }
    this.clearInputs();
    this.control = { kind: 'arm', step: 'zero', deadline: this.now() + 1000 };
    this.message = 'Arming at zero; awaiting the control response and a fresh Status read.';
    this.pump(); this.changed();
  }

  stop() {
    this.localArm = false; this.clearInputs();
    if (this.phase !== 'ready') { void this.close('STOP requested during connection'); return; }
    if (this.control?.kind === 'stop') return;
    this.control = { kind: 'stop', step: 'write', deadline: this.now() + 1000 };
    this.message = 'STOP requested; confirmation pending.';
    this.pump(); this.changed();
  }

  tick() {
    if (!this.checkDeadlines()) return;
    if (this.phase === 'ready') this.lastTick = this.now();
    this.pump(); this.changed();
  }

  checkDeadlines() {
    const now = this.now();
    if (this.phase === 'closing') {
      if (now >= this.cleanup.deadline) { this.finishClose(); return false; }
      return true;
    }
    if (this.phase === 'connecting' && now >= this.setupDeadline) { void this.close('Connection setup timed out'); return false; }
    if (!['connecting', 'ready'].includes(this.phase)) return true;
    if (!this.active()) { void this.inactive(); return false; }
    if (!this.checkPhysical()) return false;
    if (this.phase !== 'ready') return true;
    if (now - this.lastTick >= 200 || now - this.lastSubmit >= 200) { void this.close('Stream delayed by 200 ms; connect and ARM again'); return false; }
    if (this.control && now >= this.control.deadline) { void this.close('Control confirmation timed out'); return false; }
    if (this.busy?.kind === 'read' && now - this.busy.started >= 1000) { void this.close('Status read timed out'); return false; }
    return true;
  }

  // One in-flight procedure; all delayed results are checked before they can change state.
  job(kind, operation, success, failure = error => { void this.close(error.message); }) {
    const token = { generation: this.generation, kind, started: this.now() };
    this.busy = token;
    Promise.resolve().then(() => {
      if (token.generation === this.generation && this.checkDeadlines()) return operation();
    }).then(value => {
      if (token.generation === this.generation && this.checkDeadlines()) success(value);
    }).catch(error => {
      if (token.generation === this.generation) failure(error);
    }).finally(() => {
      if (this.busy === token) this.busy = null;
      this.pump(); this.changed();
    });
  }

  async sendZero(transport = this.transport) { return this.send(ZERO, transport); }
  async send(motion, transport = this.transport) {
    const bytes = setpoint(this.seq, motion); this.seq = (this.seq + 1) >>> 0;
    const started = this.now(), stats = this.stats;
    if (stats.writes) stats.maxGap = Math.max(stats.maxGap, started - this.lastSubmit);
    this.lastSubmit = started; this.nextSend = started + 50;
    await transport.send(bytes);
    stats.writes++; stats.maxWrite = Math.max(stats.maxWrite, this.now() - started);
  }

  pump() {
    if (!this.checkDeadlines() || this.busy || this.phase !== 'ready') return;
    const control = this.control;
    if (control) {
      if (this.now() >= control.deadline) { void this.close('Control confirmation timed out'); return; }
      const current = action => { if (this.control === control) action(); };
      if (control.step === 'zero') this.job('setpoint', () => this.sendZero(), () => current(() => { control.step = 'write'; }));
      else if (control.step === 'write') this.job('control', () => this.transport.control(control.kind === 'arm' ? 1 : 2), () => current(() => { control.step = 'read'; }));
      else {
        this.lastRead = this.now();
        this.job('read', () => this.transport.read(), bytes => current(() => {
          const report = this.observe(bytes);
          if (this.now() >= control.deadline) throw new Error('Late control confirmation');
          const confirmed = report.error === 'none' && (control.kind === 'arm' ? report.state === 'Armed' : report.state === 'Stopped' && report.reason === 'Stop');
          if (!confirmed) throw new Error(`${control.kind.toUpperCase()} was not confirmed by the fresh Status read`);
          this.control = null; this.localArm = control.kind === 'arm'; this.clearInputs();
          this.message = this.localArm ? (this.mode === 'physical' ? 'Armed at zero. A new phone drive-pad hold is required.' : 'Armed at zero. A new direction press is required.') : 'Car reported Stopped/Stop. ARM again before moving.';
        }));
      }
      return;
    }
    if (this.now() >= this.nextSend) this.job('setpoint', () => this.send(this.requested), () => {});
    else if (this.now() - this.lastRead >= 500) {
      this.lastRead = this.now(); this.job('read', () => this.transport.read(), bytes => this.observe(bytes));
    }
  }

  inactive() { return this.close('Page inactive; connect and ARM again before moving'); }

  close(reason = 'Disconnect requested') {
    if (this.phase === 'closing') return this.closePromise;
    this.physical.disable('Session ended; enable sensors and calibrate again.');
    if (this.phase === 'disconnected') return Promise.resolve();
    ++this.generation; this.localArm = false; this.clearInputs(); this.control = null;
    this.phase = 'closing'; this.message = reason;
    this.cleanup = { deadline: this.now() + 500, stopReported: false };
    this.closePromise = new Promise(resolve => { this.closeDone = resolve; });
    this.beginCleanup(); this.changed();
    return this.closePromise;
  }

  beginCleanup() {
    const cleanup = this.cleanup, transport = this.transport, generation = this.generation;
    if (!transport?.connected || !transport.writesReady) { this.finishClose(); return; }
    const current = () => generation === this.generation && this.phase === 'closing' && this.now() < cleanup.deadline;
    // Shutdown cannot wait for a stuck write. STOP is an independent best-effort attempt;
    // a host that rejects overlapping work leaves it unconfirmed until disconnect/lease expiry.
    const zero = this.busy ? Promise.resolve() : Promise.resolve().then(() => { if (current()) return this.sendZero(transport); });
    const stop = Promise.resolve().then(async () => {
      if (!current()) return;
      await transport.control(2);
      if (!current()) return;
      const bytes = await transport.read();
      if (!current()) return;
      const report = status(bytes); this.reported = report; this.reportedAt = this.now();
      cleanup.stopReported = report.state === 'Stopped' && report.reason === 'Stop' && report.error === 'none';
    });
    void Promise.allSettled([zero, stop]).then(() => {
      if (generation === this.generation && this.phase === 'closing') this.finishClose();
    });
  }

  finishClose() {
    const stop = this.cleanup.stopReported ? 'Device reported Stopped/Stop.' : 'STOP remains unconfirmed.';
    let ended = 'Local connection ended; physical stop/release not verified.';
    try { this.transport?.disconnect(); } catch { ended = 'Local disconnect attempt failed; stop/release unconfirmed.'; }
    ++this.generation; this.busy = null; this.cleanup = null; this.phase = 'disconnected';
    this.message += ` ${stop} ${ended}`;
    this.closeDone(); this.changed();
  }
}
