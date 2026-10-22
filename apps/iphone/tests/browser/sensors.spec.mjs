import { test, expect } from '@playwright/test';

async function setup(page, denied = false) {
  await page.addInitScript(({ denied }) => {
    window.bleAccesses = 0;
    Object.defineProperty(navigator, 'bluetooth', { configurable: true, get() { window.bleAccesses++; throw Error('Sensor test must never access BLE'); } });
    window.DeviceOrientationEvent = { requestPermission: () => Promise.resolve(denied ? 'denied' : 'granted') };
    window.DeviceMotionEvent = { requestPermission: () => Promise.resolve(denied ? 'denied' : 'granted') };
    // Headless Chromium emits real null sensor events; this fixture owns both sources.
    for (const name of ['deviceorientation', 'devicemotion']) addEventListener(name, event => {
      if (event.isTrusted) event.stopImmediatePropagation();
    }, true);
    window.angles = { alpha: 350, beta: 10, gamma: 5 }; window.emitOrientation = true;
    setInterval(() => {
      if (window.emitOrientation) dispatchEvent(Object.assign(new Event('deviceorientation'), window.angles, { absolute: false }));
      dispatchEvent(Object.assign(new Event('devicemotion'), { accelerationIncludingGravity: { x: 0, y: 0, z: 9.8 }, rotationRate: { alpha: 0, beta: 0, gamma: 0 }, interval: 20 }));
    }, 20);
  }, { denied });
  await page.goto('/sensor-test.html');
  await page.getByRole('button', { name: 'Enable sensors', exact: true }).click();
}
async function calibrated(page) {
  await expect(page.locator('#calibrate')).toBeEnabled(); await page.locator('#calibrate').click();
}
async function hold(page) {
  await page.locator('#hold').scrollIntoViewIfNeeded();
  const box = await page.locator('#hold').boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
}
const zeros = 'Forward 0% · Left 0% · Rotate left 0%';
test('denied permissions fail closed and diagnostic imports no controller modules or Bluetooth', async ({ page }) => {
  await setup(page, true);
  await expect(page.locator('#permissions')).toContainText('orientation: denied');
  await expect(page.locator('#calibrate')).toBeDisabled(); await expect(page.locator('#hold')).toBeDisabled();
  expect(await page.evaluate(() => window.bleAccesses)).toBe(0);
  const assets = await page.evaluate(() => performance.getEntriesByType('resource').map(entry => new URL(entry.name).pathname));
  expect(assets.some(path => /\/(app|session|transport|wire|hold|ui)(?:\.[^/]+)?\.js$/.test(path))).toBe(false);
});
test('three-axis preview holds a steady heading, releases to zero, and reports version and fields', async ({ page }) => {
  await setup(page); await calibrated(page); await hold(page);
  await page.evaluate(() => { window.angles = { alpha: 10, beta: -10, gamma: -15 }; });
  await expect(page.locator('#preview')).toHaveText('Forward 100% · Left 100% · Rotate left 100%');
  await page.waitForTimeout(350);
  await expect(page.locator('#preview')).toHaveText('Forward 100% · Left 100% · Rotate left 100%');
  await page.mouse.up(); await expect(page.locator('#preview')).toHaveText(zeros);
  await page.locator('#disable').click();
  const report = JSON.parse(await page.locator('#report').inputValue());
  const revision = await page.locator('meta[name="rovi-build"]').getAttribute('content');
  if (revision !== 'source-checkout') expect(report.revision).toBe(revision);
  expect(report.diagnosticOnly).toBe(true); expect(report.fields['motion.rotationAlpha'].valid).toBeGreaterThan(0);
  expect(await page.evaluate(() => window.bleAccesses)).toBe(0);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});
test('sensor expiry clears all axes while UI and motion events continue; new data cannot replay', async ({ page }) => {
  await setup(page); await calibrated(page); await hold(page);
  await page.evaluate(() => { window.angles.alpha = 10; });
  await expect(page.locator('#preview')).toContainText('Rotate left 100%');
  await page.evaluate(() => { window.emitOrientation = false; });
  await expect(page.locator('#preview')).toHaveText(zeros);
  await page.evaluate(() => { window.emitOrientation = true; });
  await expect(page.locator('#hold')).toBeDisabled(); await expect(page.locator('#preview')).toHaveText(zeros);
  await page.mouse.up();
});
test('scheduled UI stall expires the hold independently of fresh sensor input', async ({ page }) => {
  await setup(page); await calibrated(page); await page.locator('#freeze-ui').click(); await hold(page);
  await page.evaluate(() => { window.angles.alpha = 10; });
  await expect(page.locator('#preview')).toContainText('Rotate left 100%');
  await expect(page.locator('#message')).toContainText('UI renewal expired', { timeout: 5000 });
  await expect(page.locator('#preview')).toHaveText(zeros); await page.mouse.up();
  await page.locator('#freeze-ui').click(); await expect(page.locator('#hold')).toBeDisabled();
});
test('blur invalidates collection; focus and continuing synthetic events cannot resume it', async ({ page }) => {
  await setup(page); await calibrated(page); await hold(page);
  await page.evaluate(() => { window.angles.alpha = 10; });
  await expect(page.locator('#preview')).toContainText('Rotate left 100%');
  await page.evaluate(() => { dispatchEvent(new Event('blur')); dispatchEvent(new Event('focus')); });
  await expect(page.locator('#preview')).toHaveText(zeros); await expect(page.locator('#calibrate')).toBeDisabled();
  await expect(page.locator('#hold')).toBeDisabled(); await page.mouse.up();
});
