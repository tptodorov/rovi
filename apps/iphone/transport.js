import { SERVICE, UUID, NETWORK_UUID } from './wire.js';

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

  constructor(device, onStatus, onLost, onNetwork = () => {}) {
    this.device = device;
    this.onStatus = event => { if (!this.closed) onStatus(copy(event.target.value)); };
    this.onNetwork = event => { if (!this.closed) onNetwork(copy(event.target.value)); };
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
    try { this.chars.network = await service.getCharacteristic(NETWORK_UUID); } catch (error) {
      if (error.name !== 'NotFoundError') throw error;
    }
    this.check();
    if (!this.chars.network?.properties.read || typeof this.chars.network.readValue !== 'function') this.chars.network = null;
    const { setpoint, control, capabilities, status } = this.chars;
    if (!setpoint.properties.writeWithoutResponse || typeof setpoint.writeValueWithoutResponse !== 'function'
      || !control.properties.write || typeof control.writeValueWithResponse !== 'function'
      || !capabilities.properties.read || typeof capabilities.readValue !== 'function'
      || !status.properties.read || !status.properties.notify || typeof status.readValue !== 'function'
      || typeof status.startNotifications !== 'function') throw new Error('Unsupported product GATT properties or browser write/read APIs');
    this.writesReady = true;
    const caps = copy(await capabilities.readValue()); this.check();
    // Claim with an acknowledged STOP: ADR-0016 firmware claims on the first write (a bare
    // connection only watches), and the response tells a claim from a car another controller
    // owns. Older firmware has already claimed on connect; STOP keeps it disarmed either way.
    try { await control.writeValueWithResponse(Uint8Array.of(2)); } catch (error) {
      this.check(); throw new Error(`Car busy: another controller owns the car (${error.message})`);
    }
    this.check();
    const report = copy(await status.readValue()); this.check();
    status.addEventListener('characteristicvaluechanged', this.onStatus);
    await status.startNotifications(); this.check();
    const network = await this.readNetwork();
    if (this.chars.network?.properties.notify && typeof this.chars.network.startNotifications === 'function') {
      this.chars.network.addEventListener('characteristicvaluechanged', this.onNetwork);
      await this.chars.network.startNotifications(); this.check();
    }
    return { caps, report, network };
  }

  async send(bytes) { this.check(); await this.chars.setpoint.writeValueWithoutResponse(bytes); this.check(); }
  async control(code) { this.check(); await this.chars.control.writeValueWithResponse(Uint8Array.of(code)); this.check(); }
  async read() { this.check(); const result = copy(await this.chars.status.readValue()); this.check(); return result; }
  async readNetwork() {
    this.check();
    if (!this.chars.network) return null;
    const result = copy(await this.chars.network.readValue()); this.check(); return result;
  }

  disconnect() {
    this.closed = true;
    this.chars.status?.removeEventListener('characteristicvaluechanged', this.onStatus);
    this.chars.network?.removeEventListener('characteristicvaluechanged', this.onNetwork);
    this.device.removeEventListener('gattserverdisconnected', this.onLost);
    this.device.gatt?.disconnect();
  }
}
