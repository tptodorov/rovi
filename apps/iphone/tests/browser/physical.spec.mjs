import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.permissionResult = 'granted'; window.permissionCalls = 0; window.emitAngles = true; window.holdFrames = true;
    window.DeviceOrientationEvent = { requestPermission: () => {
      window.permissionCalls++;
      return window.permissionResult === 'pending' ? new Promise(resolve => { window.resolvePermission = resolve; }) : Promise.resolve(window.permissionResult);
    } };
    addEventListener('deviceorientation', event => { if (event.isTrusted) event.stopImmediatePropagation(); }, true);
    window.phoneAngles = { alpha: 350, beta: 0, gamma: 0 };
    setInterval(() => { if (window.emitAngles) dispatchEvent(Object.assign(new Event('deviceorientation'), window.phoneAngles)); }, 20);
    const raf = window.requestAnimationFrame.bind(window);
    window.requestAnimationFrame = callback => raf(function next(time) { if (window.holdFrames) callback(time); else raf(next); });
  });
  await page.route('**/fake-ble.js', async route => route.fulfill({ contentType: 'text/javascript', body: await readFile(new URL('../fake-ble.js', import.meta.url), 'utf8') }));
  await page.route('**/app*.js*', async route => route.fulfill({ contentType: 'text/javascript', body: `
import { fakeBle } from './fake-ble.js';
window.fake = fakeBle();
Object.defineProperty(navigator, 'bluetooth', { value: fake.bluetooth, configurable: true });
${await readFile(new URL('../../app.js', import.meta.url), 'utf8')}` }));
  await page.goto('/');
});
async function ready(page) {
  await page.locator('#connect').click(); await expect(page.locator('#phase')).toHaveText('ready');
  await page.locator('#control-mode').selectOption('physical');
  await page.locator('#sensors-enable').click();
  await expect(page.locator('#physical-calibrate')).toBeEnabled(); await page.locator('#physical-calibrate').click();
  await page.locator('#arm').click(); await expect(page.locator('#local-arm')).toHaveText('Locally armed');
}
async function hold(page) {
  await page.locator('#physical-drive').scrollIntoViewIfNeeded();
  const box = await page.locator('#physical-drive').boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
}
const zero = 'vx 0.00 · vy 0.00 · wz 0.00';
test('default Buttons never prompt; Physical combines angles and switching while held stops/disarms', async ({ page }, info) => {
  await expect(page.locator('#control-mode')).toHaveValue('buttons');
  expect(await page.evaluate(() => window.permissionCalls)).toBe(0);
  await ready(page); await hold(page);
  const revision = await page.locator('meta[name="rovi-build"]').getAttribute('content');
  if (revision !== 'source-checkout') await expect(page.locator('#version')).toHaveText(`Build ${revision}`);
  await page.evaluate(() => { window.phoneAngles = { alpha: 10, beta: -20, gamma: -20 }; });
  await expect(page.locator('#requested')).toHaveText('vx 0.10 · vy 0.10 · wz 0.30');
  await page.waitForTimeout(350); await expect(page.locator('#requested')).toContainText('wz 0.30');
  await page.screenshot({ path: info.outputPath('physical.png'), fullPage: true });
  await page.locator('#control-mode').selectOption('buttons');
  await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
  await expect(page.locator('#reported')).toContainText('Stopped · reason Stop');
  await expect(page.locator('#requested')).toHaveText(zero); await page.mouse.up();
});
test('permission denial and late permission after mode switch cannot arm or enable Physical', async ({ page }) => {
  await page.locator('#connect').click(); await expect(page.locator('#phase')).toHaveText('ready');
  await page.evaluate(() => { window.permissionResult = 'denied'; });
  await page.locator('#control-mode').selectOption('physical'); await page.locator('#sensors-enable').click();
  await expect(page.locator('#physical-status')).toContainText('denied'); await expect(page.locator('#arm')).toBeDisabled();
  await page.evaluate(() => { window.permissionResult = 'pending'; }); await page.locator('#sensors-enable').click();
  await page.locator('#control-mode').selectOption('buttons');
  await page.evaluate(() => window.resolvePermission('granted'));
  await page.locator('#control-mode').selectOption('physical');
  await expect(page.locator('#physical-calibrate')).toBeDisabled();
  await expect(page.locator('#physical-reading')).toHaveText('No sensor reading');
  expect(await page.evaluate(() => fake.calls.some(([kind, value]) => kind === 'control' && value === 1))).toBe(false);
});
test('sensor or UI renewal loss independently ends Physical; returning data does not recover', async ({ page }) => {
  for (const source of ['emitAngles', 'holdFrames']) {
    await ready(page); await hold(page);
    await page.evaluate(() => { window.phoneAngles = { alpha: 10, beta: -20, gamma: -20 }; });
    await expect(page.locator('#requested')).toContainText('vx 0.10');
    await page.evaluate(key => { window[key] = false; }, source);
    await expect(page.locator('#phase')).toHaveText('disconnected');
    await page.evaluate(key => { window[key] = true; }, source); await page.mouse.up();
    await expect(page.locator('#requested')).toHaveText(zero); await expect(page.locator('#physical-calibrate')).toBeDisabled();
    await page.reload();
  }
});
test('screen orientation change while held closes and cannot reconnect automatically', async ({ page }) => {
  await ready(page); await hold(page);
  await page.evaluate(() => { window.phoneAngles.alpha = 10; });
  await expect(page.locator('#requested')).toContainText('wz 0.30');
  await page.evaluate(() => dispatchEvent(new Event('orientationchange')));
  await expect(page.locator('#phase')).toHaveText('disconnected'); await page.mouse.up();
  expect(await page.evaluate(() => fake.calls.filter(call => call[0] === 'picker').length)).toBe(1);
});
test('a second finger cannot start Physical while the first finger is outside the pad', async ({ page }) => {
  await ready(page); await page.evaluate(() => { window.phoneAngles.alpha = 10; });
  const box = await page.locator('#physical-drive').boundingBox();
  const cdp = await page.context().newCDPSession(page);
  const outside = { x: 2, y: 2, id: 1 }, inside = { x: box.x + box.width / 2, y: box.y + box.height / 2, id: 2 };
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [outside] });
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [outside, inside] });
  await expect(page.locator('#requested')).toHaveText(zero);
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
});
