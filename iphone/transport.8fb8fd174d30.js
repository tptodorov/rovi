import { SERVICE, UUID } from './wire.8fb8fd174d30.js';

export function supportError(bluetooth, protocol) {
  if (protocol !== 'https:') return 'Open the HTTPS controller page in Bluefy.';
  if (typeof bluetooth?.requestDevice !== 'function') return 'Bluetooth web access is unavailable. Open this page in Bluefy and allow Bluetooth.';
  return '';
}

export function pickDevice(bluetooth, protocol) {
  const error = supportError(bluetooth, protocol);
  if (error) throw new Error(error);
  return bluetooth.requestDevice({ filters: [{ name: 'Rovi M8' }], optionalServices: [SERVICE] });
}

const copy = data => new Uint8Array(data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength));

export class Ble {
  closed = false;
  writesReady = false;
  chars = {};

  constructor(device, onStatus, onLost) {
    this.device = device;
    this.onStatus = event => { if (!this.closed) onStatus(copy(event.target.value)); };
    this.onLost = () => { if (!this.closed) onLost(); };
    device.addEventListener('gattserverdisconnected', this.onLost);
  }

  get connected() { return !this.closed && Boolean(this.device.gatt?.connected); }

  check() { if (!this.connected) throw new Error('Bluetooth connection ended'); }

  async open() {
    if (!this.device.gatt) throw new Error('Selected device has no GATT connection');
    const server = await this.device.gatt.connect();
    if (this.closed) { server.disconnect(); throw new Error('Connect completed after cancellation'); }
    this.check();
    const service = await server.getPrimaryService(SERVICE); this.check();
    for (const [name, uuid] of Object.entries(UUID)) {
      this.chars[name] = await service.getCharacteristic(uuid); this.check();
    }
    const { setpoint, control, capabilities, status } = this.chars;
    if (!setpoint.properties.writeWithoutResponse || typeof setpoint.writeValueWithoutResponse !== 'function'
      || !control.properties.write || typeof control.writeValueWithResponse !== 'function'
      || !capabilities.properties.read || typeof capabilities.readValue !== 'function'
      || !status.properties.read || !status.properties.notify || typeof status.readValue !== 'function'
      || typeof status.startNotifications !== 'function') throw new Error('Unsupported product GATT properties or browser write/read APIs');
    this.writesReady = true;
    const caps = copy(await capabilities.readValue()); this.check();
    const report = copy(await status.readValue()); this.check();
    status.addEventListener('characteristicvaluechanged', this.onStatus);
    await status.startNotifications(); this.check();
    return { caps, report };
  }

  async send(bytes) { this.check(); await this.chars.setpoint.writeValueWithoutResponse(bytes); this.check(); }
  async control(code) { this.check(); await this.chars.control.writeValueWithResponse(Uint8Array.of(code)); this.check(); }
  async read() { this.check(); const result = copy(await this.chars.status.readValue()); this.check(); return result; }

  disconnect() {
    this.closed = true;
    this.chars.status?.removeEventListener('characteristicvaluechanged', this.onStatus);
    this.device.removeEventListener('gattserverdisconnected', this.onLost);
    this.device.gatt?.disconnect();
  }
}
