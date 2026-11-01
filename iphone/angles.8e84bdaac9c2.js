export const angleDelta = (value, neutral) => ((value - neutral + 540) % 360 + 360) % 360 - 180;
export function scaleAngle(degrees) {
  const magnitude = Math.min(1, Math.max(0, (Math.abs(degrees) - 3) / 17));
  return Math.sign(degrees) * magnitude || 0;
}
