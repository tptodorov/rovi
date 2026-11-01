// Car messages for the phone: a thin adapter over the rovi-protocol WebAssembly module (ADR-0017).
// Layouts, validation and names live only in the crate; the module has no JavaScript fallback, so
// when it cannot load, `protocolError()` explains why and the page keeps Connect disabled.
let rp = null, loadError = null;
try {
  const module = await import('./rovi_protocol.712ef024b110.js');
  const wasm = new URL('./rovi_protocol_bg.712ef024b110.wasm', import.meta.url);
  // Node tests have no file: fetch; browsers stream the pinned asset.
  const source = wasm.protocol === 'file:' ? await (await import('node:fs/promises')).readFile(wasm) : wasm;
  await module.default({ module_or_path: source });
  rp = module;
} catch (error) {
  loadError = error;
}

export const protocolError = () => loadError && `Car protocol module could not load (${loadError.message}). Reload this page to try again.`;
const uuids = rp?.constants().bleUuids;
export const SERVICE = uuids?.service;
export const NETWORK_UUID = uuids?.network;
export const UUID = uuids && { setpoint: uuids.setpoint, control: uuids.control, capabilities: uuids.capabilities, status: uuids.status };
export const ZERO = Object.freeze({ vx: 0, vy: 0, wz: 0 });
export const DIRECTIONS = {
  forward: ['vx', 1], backward: ['vx', -1],
  left: ['vy', 1], right: ['vy', -1],
  ccw: ['wz', 1], cw: ['wz', -1],
};

const bytes = data => data instanceof DataView ? new Uint8Array(data.buffer, data.byteOffset, data.byteLength) : data;

export function setpoint(seq, { vx = 0, vy = 0, wz = 0 } = ZERO) {
  return rp.bleSetpointEncode(seq, { vx, vy, wz });
}

export function capabilities(data) {
  const caps = rp.capabilitiesDecode(bytes(data));
  if (!rp.meetsClientMinimum(caps)) throw new Error('Unsupported capabilities (version, geometry, limits or windows)');
  const { wheels_radius: radius, center_projection_sum: projection, ...rest } = caps;
  return Object.freeze({ ...rest, radius, projection });
}

export function status(data) {
  const { state, reason, error } = rp.statusDecode(bytes(data));
  return { state, reason, error };
}

export function direction(caps, name) {
  const [axis, sign] = DIRECTIONS[name] ?? [];
  if (!axis) throw new Error('Unknown direction');
  return { ...ZERO, [axis]: sign * Math.min(axis === 'wz' ? 0.3 : 0.1, caps[`max_${axis}`]) };
}

export function network(data) {
  return rp.networkDecode(bytes(data));
}

// The Control characteristic byte for `arm`, `stop`, `home` or `car-network`.
export const controlCode = name => rp.controlEncode(name)[0];
