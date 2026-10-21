import { ZERO, direction, DIRECTIONS } from './wire.js';

// Only UI frames renew a tracked hold. Transmission never extends this deadline.
export class Hold {
  pointers = new Map();
  current = null;
  blocked = false;
  expires = 0;

  press(id, name, allowed, now) {
    if (!DIRECTIONS[name]) return;
    this.track(id, name, allowed, now);
  }

  track(id, name, allowed, now) {
    this.pointers.set(id, name);
    if (!allowed || this.blocked || this.pointers.size !== 1) return this.clear();
    this.current = id;
    this.expires = now + 250;
  }

  release(id) {
    this.pointers.delete(id);
    this.clear();
  }

  clear() {
    this.current = null;
    this.expires = 0;
    this.blocked = this.pointers.size > 0;
  }

  reset() {
    this.pointers.clear();
    this.clear();
  }

  renew(now) {
    if (this.current === null) return;
    if (now >= this.expires) return this.clear();
    this.expires = now + 250;
  }

  intent(caps, now) {
    if (now >= this.expires) this.clear();
    return this.current === null ? ZERO : direction(caps, this.pointers.get(this.current));
  }
}
