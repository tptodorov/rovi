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
    window.orientationCallbacks = [];
    const add = window.addEventListener.bind(window);
    window.addEventListener = (name, callback, options) => {
      if (name === 'deviceorientation') window.orientationCallbacks.push(callback);
      return add(name, callback, options);
    };
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
  await expect(page.locator('#arm')).toBeEnabled();
  await page.locator('#arm').click(); await expect(page.locator('#local-arm')).toHaveText('Locally armed');
}
async function start(page) { await page.locator('#physical-drive').click(); }
const zero = 'vx 0.00 · vy 0.00 · wz 0.00';
test('tap starts tilt control without holding; tap pauses and neutral is required to restart', async ({ page }) => {
  await ready(page);
  await page.locator('#physical-drive').click();
  await page.evaluate(() => { window.phoneAngles = { alpha: 10, beta: -20, gamma: -20 }; });
  await expect(page.locator('#requested')).toHaveText('vx 0.10 · vy 0.10 · wz 0.30');
  await expect(page.locator('#physical-requested')).toContainText('vx 0.10 · vy 0.10 · wz 0.30');
  await page.waitForTimeout(350);
  await expect(page.locator('#physical-drive')).toHaveText('PAUSE TILT CONTROL');
  await page.locator('#physical-drive').click();
  await expect(page.locator('#requested')).toHaveText(zero);
  await expect(page.locator('#local-arm')).toHaveText('Locally armed');
  await expect(page.locator('#physical-drive')).toBeDisabled();
  await page.evaluate(() => { window.phoneAngles = { alpha: 350, beta: 0, gamma: 0 }; });
  await expect(page.locator('#physical-drive')).toBeEnabled();
  await expect(page.locator('#requested')).toHaveText(zero);
  await page.locator('#physical-drive').click();
  await page.evaluate(() => { window.phoneAngles.alpha = 10; });
  await expect(page.locator('#requested')).toContainText('wz 0.30');
  await page.locator('#stop').click();
  await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
  await expect(page.locator('#requested')).toHaveText(zero);
  await page.locator('#arm').click();
  await expect(page.locator('#local-arm')).toHaveText('Locally armed');
  await expect(page.locator('#physical-drive')).toHaveText('START TILT CONTROL');
  await expect(page.locator('#requested')).toHaveText(zero);
});
test('scrolling while paused preserves arm and keeps STOP and status visible', async ({ page }) => {
  const errors = []; page.on('pageerror', error => errors.push(error.message));
  await ready(page);
  await page.evaluate(() => { window.emitAngles = false; });
  await page.mouse.wheel(0, 600); await page.waitForTimeout(350);
  await expect(page.locator('#phase')).toHaveText('ready');
  await expect(page.locator('#local-arm')).toHaveText('Locally armed');
  await expect(page.locator('#requested')).toHaveText(zero);
  for (const id of ['local-arm', 'stop', 'message', 'physical-drive', 'physical-requested']) await expect(page.locator(`#${id}`)).toBeInViewport();
  await page.evaluate(() => { window.phoneAngles.beta = 75; window.emitAngles = true; });
  await expect(page.locator('#physical-reading')).toContainText('β 75.0');
  await expect(page.locator('#physical-drive')).toBeDisabled();
  await expect(page.locator('#local-arm')).toHaveText('Locally armed');
  await page.evaluate(() => { window.phoneAngles.beta = 0; });
  await expect(page.locator('#physical-drive')).toBeEnabled();
  await expect(page.locator('#requested')).toHaveText(zero);
  await page.locator('#stop').click(); await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
  expect(errors).toEqual([]);
});
test('scroll pauses running control without replay or discarding neutral', async ({ page }) => {
  await ready(page); await start(page);
  await page.evaluate(() => { window.phoneAngles.alpha = 10; });
  await expect(page.locator('#requested')).toContainText('wz 0.30');
  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  await expect(page.locator('#requested')).toHaveText(zero);
  await expect(page.locator('#local-arm')).toHaveText('Locally armed');
  await page.evaluate(() => window.scrollTo(0, 0));
  await expect(page.locator('#requested')).toHaveText(zero);
  await expect(page.locator('#physical-drive')).toBeDisabled();
  await page.evaluate(() => { window.phoneAngles.alpha = 350; });
  await start(page); await page.evaluate(() => { window.phoneAngles.alpha = 10; });
  await expect(page.locator('#requested')).toContainText('wz 0.30');
});
test('default Buttons never prompt; Physical combines angles and switching while running stops/disarms', async ({ page }, info) => {
  await expect(page.locator('#control-mode')).toHaveValue('buttons');
  expect(await page.evaluate(() => window.permissionCalls)).toBe(0);
  await ready(page); await start(page);
  const revision = await page.locator('meta[name="rovi-build"]').getAttribute('content');
  if (revision !== 'source-checkout') await expect(page.locator('#version')).toHaveText(`Build ${revision}`);
  await page.evaluate(() => { window.phoneAngles = { alpha: 10, beta: -20, gamma: -20 }; });
  await expect(page.locator('#requested')).toHaveText('vx 0.10 · vy 0.10 · wz 0.30');
  await page.waitForTimeout(350); await expect(page.locator('#requested')).toContainText('wz 0.30');
  await page.screenshot({ path: info.outputPath('physical.png'), fullPage: true });
  await page.locator('#control-mode').selectOption('buttons');
  await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
  await expect(page.locator('#reported')).toContainText('Stopped · reason Stop');
  await expect(page.locator('#requested')).toHaveText(zero);
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
  await expect(page.locator('#arm')).toBeDisabled();
  await expect(page.locator('#physical-reading')).toHaveText('No sensor reading');
  expect(await page.evaluate(() => fake.calls.some(([kind, value]) => kind === 'control' && value === 1))).toBe(false);
});
test('sensor or UI renewal loss independently ends Physical; returning data does not recover', async ({ page }) => {
  for (const source of ['emitAngles', 'holdFrames']) {
    await ready(page); await start(page);
    await page.evaluate(() => { window.phoneAngles = { alpha: 10, beta: -20, gamma: -20 }; });
    await expect(page.locator('#requested')).toContainText('vx 0.10');
    await page.evaluate(key => { window[key] = false; }, source);
    await expect(page.locator('#phase')).toHaveText('disconnected');
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
    for (const id of ['local-arm', 'stop', 'message', 'physical-drive', 'physical-requested']) await expect(page.locator(`#${id}`)).toBeInViewport();
    await expect(page.locator('#message')).toContainText(source === 'emitAngles' ? 'Sensor readings expired' : 'Foreground renewal expired');
    await page.evaluate(key => { window[key] = true; }, source);
    await expect(page.locator('#requested')).toHaveText(zero); await expect(page.locator('#arm')).toBeDisabled();
    await page.reload();
  }
});
test('screen orientation change while running closes and cannot reconnect automatically', async ({ page }) => {
  await ready(page); await start(page);
  await page.evaluate(() => { window.phoneAngles.alpha = 10; });
  await expect(page.locator('#requested')).toContainText('wz 0.30');
  await page.evaluate(() => dispatchEvent(new Event('orientationchange')));
  await expect(page.locator('#phase')).toHaveText('disconnected');
  expect(await page.evaluate(() => fake.calls.filter(call => call[0] === 'picker').length)).toBe(1);
});
test('a detached sensor callback cannot replace a newly calibrated and armed input', async ({ page }) => {
  await ready(page); await page.locator('#control-mode').selectOption('buttons');
  await page.locator('#control-mode').selectOption('physical');
  await page.evaluate(() => { window.phoneAngles = { alpha: 123, beta: 0, gamma: 0 }; });
  await page.locator('#sensors-enable').click();
  await expect(page.locator('#physical-reading')).toContainText('α 123.0');
  await page.locator('#arm').click();
  await expect(page.locator('#local-arm')).toHaveText('Locally armed'); await start(page);
  await page.evaluate(() => { window.phoneAngles.alpha = 143; });
  await expect(page.locator('#requested')).toContainText('wz 0.30');
  const actual = await page.evaluate(() => {
    window.orientationCallbacks[0]({ alpha: 103, beta: 0, gamma: 0 });
    return document.getElementById('requested').textContent;
  });
  expect(actual).toContain('wz 0.30');
});
test('keyboard activation explicitly starts and pauses Physical control', async ({ page }) => {
  await ready(page); await page.locator('#physical-drive').focus();
  await page.keyboard.press('Space');
  await expect(page.locator('#physical-drive')).toHaveAttribute('aria-pressed', 'true');
  await page.evaluate(() => { window.phoneAngles.alpha = 10; });
  await expect(page.locator('#requested')).toContainText('wz 0.30');
  await page.keyboard.press('Space');
  await expect(page.locator('#requested')).toHaveText(zero);
  await expect(page.locator('#physical-drive')).toHaveAttribute('aria-pressed', 'false');
});
test('rotation posture fault names the offset and limit and cannot auto-recover', async ({ page }) => {
  await ready(page); await start(page);
  await page.evaluate(() => { window.phoneAngles.alpha = 100; });
  await expect(page.locator('#phase')).toHaveText('disconnected');
  await expect(page.locator('#message')).toContainText('Rotation offset 110.0° exceeds ±90°');
  await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
  await expect(page.locator('#requested')).toHaveText(zero);
  await page.evaluate(() => { window.phoneAngles.alpha = 350; });
  await expect(page.locator('#phase')).toHaveText('disconnected');
});
test('Physical blur or hidden page clears running state and cannot replay after return', async ({ page }) => {
  for (const name of ['blur', 'visibilitychange']) {
    await ready(page); await start(page);
    await page.evaluate(() => { window.phoneAngles.alpha = 10; });
    await expect(page.locator('#requested')).toContainText('wz 0.30');
    await page.evaluate(event => {
      if (event === 'visibilitychange') Object.defineProperty(document, 'hidden', { value: true, configurable: true });
      (event === 'blur' ? window : document).dispatchEvent(new Event(event));
      if (event === 'visibilitychange') delete document.hidden;
    }, name);
    await expect(page.locator('#phase')).toHaveText('disconnected');
    await page.evaluate(() => dispatchEvent(new Event('focus')));
    await expect(page.locator('#requested')).toHaveText(zero);
    await expect(page.locator('#physical-drive')).toHaveAttribute('aria-pressed', 'false');
    await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
    expect(await page.evaluate(() => fake.calls.filter(call => call[0] === 'picker').length)).toBe(1);
    await page.reload();
  }
});
