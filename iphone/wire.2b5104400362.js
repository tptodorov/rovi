export const SERVICE = 'd47a0010-45b2-4d19-8db0-6bd87e9c0001';
export const UUID = Object.fromEntries(['setpoint', 'control', 'capabilities', 'status'].map(
  (name, i) => [name, `d47a001${i + 1}-45b2-4d19-8db0-6bd87e9c0001`],
));
export const ZERO = Object.freeze({ vx: 0, vy: 0, wz: 0 });
export const DIRECTIONS = {
  forward: ['vx', 1], backward: ['vx', -1],
  left: ['vy', 1], right: ['vy', -1],
  ccw: ['wz', 1], cw: ['wz', -1],
};

function view(data, length) {
  const bytes = data instanceof DataView ? data : new DataView(data.buffer, data.byteOffset, data.byteLength);
  if (bytes.byteLength !== length) throw new Error(`Expected ${length} bytes, received ${bytes.byteLength}`);
  return bytes;
}

export function setpoint(seq, { vx = 0, vy = 0, wz = 0 } = ZERO) {
  if (!Number.isInteger(seq) || seq < 0 || seq > 0xffffffff || ![vx, vy, wz].every(Number.isFinite)) {
    throw new Error('Invalid sequence or non-finite setpoint');
  }
  const bytes = new Uint8Array(72), out = new DataView(bytes.buffer);
  out.setUint32(0, seq, true);
  bytes.set([0, 1, 0, 0], 4); // CDR little-endian encapsulation.
  out.setUint32(16, 1, true); // Empty frame string including NUL; timestamp and padding stay zero.
  out.setFloat64(24, vx, true);
  out.setFloat64(32, vy, true);
  out.setFloat64(64, wz, true);
  return bytes;
}

export function capabilities(data) {
  const bytes = view(data, 27), caps = { version: bytes.getUint8(0) };
  ['radius', 'projection', 'max_vx', 'max_vy', 'max_wz'].forEach((key, i) => { caps[key] = bytes.getFloat32(1 + 4 * i, true); });
  ['lease_ms', 'claim_ms', 'reclaim_ms'].forEach((key, i) => { caps[key] = bytes.getUint16(21 + 2 * i, true); });
  if (caps.version !== 1 || ![caps.radius, caps.projection].every(x => Number.isFinite(x) && x > 0)
    || ![caps.max_vx, caps.max_vy, caps.max_wz].every(x => Number.isFinite(x) && x >= 0)
    || caps.lease_ms < 500 || caps.claim_ms < 1000 || caps.reclaim_ms === 0) {
    throw new Error('Unsupported capabilities (version, geometry, limits or windows)');
  }
  return Object.freeze(caps);
}

export function status(data) {
  const bytes = view(data, 3), [state, reason, error] = [0, 1, 2].map(i => bytes.getUint8(i));
  if (state > 3 || reason > 4 || error > 4 || (state === 3 ? reason === 0 : reason !== 0)) {
    throw new Error('Invalid device Status');
  }
  return {
    state: ['Open', 'Claimed', 'Armed', 'Stopped'][state],
    reason: ['none', 'Stop', 'LeaseExpired', 'OwnerLost', 'OutOfLimits'][reason],
    error: ['none', 'Busy', 'NotOwner', 'NotReady', 'Invalid'][error],
  };
}

export function direction(caps, name) {
  const [axis, sign] = DIRECTIONS[name] ?? [];
  if (!axis) throw new Error('Unknown direction');
  return { ...ZERO, [axis]: sign * Math.min(axis === 'wz' ? 0.3 : 0.1, caps[`max_${axis}`]) };
}
