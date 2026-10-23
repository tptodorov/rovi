import { UUID, SERVICE } from '../wire.js';

export const capsBytes = () => Uint8Array.from('018fc2f53c5c8f023e3333333f9a99993e00004040e80310278813'.match(/../g).map(x => Number.parseInt(x, 16)));
export const pending = () => { let resolve, reject; const promise = new Promise((yes, no) => { resolve = yes; reject = no; }); return { promise, resolve, reject }; };

// claimOnWrite: ADR-0016 firmware, where connecting only watches and the first write claims.
// busy: another controller owns the car, so a claiming control write is refused.
export function fakeBle({ claimOnWrite = false, busy = false } = {}) {
  const calls = [], device = new EventTarget();
  device.name = 'Fake Rovi'; device.id = 'fake-only';
  let report = Uint8Array.of(claimOnWrite ? (busy ? 2 : 0) : 1, 0, 0), fresh = false;
  const chars = Object.fromEntries(Object.keys(UUID).map(name => [name, new EventTarget()]));
  for (const [name, char] of Object.entries(chars)) char.uuid = UUID[name];
  chars.setpoint.properties = { writeWithoutResponse: true };
  chars.control.properties = { write: true };
  chars.capabilities.properties = { read: true };
  chars.status.properties = { read: true, notify: true };
  const read = data => new DataView(Uint8Array.from(data).buffer);
  chars.capabilities.readValue = async () => read(capsBytes());
  chars.status.readValue = async () => { calls.push(['read']); return read(report); };
  chars.status.startNotifications = async () => { calls.push(['notify']); };
  chars.setpoint.writeValueWithoutResponse = async bytes => { calls.push(['setpoint', Array.from(bytes)]); fresh = true; };
  chars.control.writeValueWithResponse = async bytes => {
    const code = bytes[0]; calls.push(['control', code]);
    if (busy) throw new Error('NotAllowed: another controller owns the car');
    if (code === 1 && !fresh) throw new Error('NotReady');
    report = code === 1 ? Uint8Array.of(2, 0, 0) : Uint8Array.of(3, 1, 0);
    fresh = false;
    notify(report);
  };
  function notify(bytes) {
    report = Uint8Array.from(bytes); chars.status.value = read(report);
    chars.status.dispatchEvent(new Event('characteristicvaluechanged'));
  }
  const server = {
    connected: false,
    async connect() { calls.push(['connect']); this.connected = true; return this; },
    async getPrimaryService(uuid) {
      if (uuid !== SERVICE) throw new Error('Wrong service');
      return { async getCharacteristic(id) { return Object.values(chars).find(x => x.uuid === id); } };
    },
    disconnect() { calls.push(['disconnect']); this.connected = false; device.dispatchEvent(new Event('gattserverdisconnected')); },
  };
  device.gatt = server;
  const bluetooth = { async requestDevice(options) { calls.push(['picker', options]); return device; } };
  return { bluetooth, device, server, chars, calls, notify };
}
