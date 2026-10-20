import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';

test.beforeEach(async ({ page }) => {
  await page.route('**/fake-ble.js', async route => route.fulfill({
    contentType: 'text/javascript', body: await readFile(new URL('../fake-ble.js', import.meta.url), 'utf8'),
  }));
  await page.route('**/app.js', async route => route.fulfill({ contentType: 'text/javascript', body: `
import { fakeBle } from './fake-ble.js';
const fake = fakeBle();
Object.defineProperty(navigator, 'bluetooth', { value: fake.bluetooth, configurable: true });
window.fake = fake;
${await readFile(new URL('../../app.js', import.meta.url), 'utf8')}
` }));
  await page.goto('/');
});

async function connect(page) {
  await page.getByRole('button', { name: 'Choose & connect' }).click();
  await expect(page.locator('#phase')).toHaveText('ready');
  await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
}

test('real page and session: connect disarmed, explicit ARM, hold and release, confirmed STOP', async ({ page }) => {
  const errors = []; page.on('pageerror', error => errors.push(error.message));
  await connect(page);
  expect(await page.evaluate(() => fake.calls.some(([kind]) => kind === 'control'))).toBe(false);
  await page.getByRole('button', { name: 'ARM AT ZERO' }).click();
  await expect(page.locator('#local-arm')).toHaveText('Locally armed');
  const button = page.getByRole('button', { name: 'Hold to move forward', exact: true });
  const box = await button.boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down();
  await expect.poll(() => page.evaluate(() => fake.calls.filter(x => x[0] === 'setpoint').some(x => new DataView(Uint8Array.from(x[1]).buffer).getFloat64(24, true) === 0.1))).toBe(true);
  await page.mouse.up();
  await expect(page.locator('#requested')).toContainText('vx 0.00');
  await page.getByRole('button', { name: 'STOP', exact: true }).click();
  await expect(page.locator('#reported')).toContainText('Stopped · reason Stop');
  await expect(button).toBeDisabled();
  await expect(page.locator('#message')).toContainText('Car reported Stopped/Stop');
  expect(errors).toEqual([]);
});

test('multi-touch cancels intent until every old pointer is released', async ({ page }) => {
  await connect(page); await page.getByRole('button', { name: 'ARM AT ZERO' }).click();
  const forward = await page.locator('[data-direction="forward"]').boundingBox();
  const left = await page.locator('[data-direction="left"]').boundingBox();
  const cdp = await page.context().newCDPSession(page);
  const point = (box, id) => ({ x: box.x + box.width / 2, y: box.y + box.height / 2, id });
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [point(forward, 1)] });
  await expect(page.locator('#requested')).toContainText('vx 0.10');
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [point(forward, 1), point(left, 2)] });
  await expect(page.locator('#requested')).toContainText('vx 0.00');
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [point(forward, 1)] });
  await expect(page.locator('#requested')).toContainText('vx 0.00');
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [point(left, 3)] });
  await expect(page.locator('#requested')).toContainText('vy 0.10');
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
});

test('phone-sized preview, real Status age and blur never recover ownership automatically', async ({ page }, info) => {
  await connect(page);
  await expect(page.locator('#reported')).toContainText('Claimed');
  const backward = await page.locator('[data-direction="backward"]').boundingBox();
  expect(backward.y + backward.height).toBeLessThanOrEqual(844);
  await page.screenshot({ path: info.outputPath('controller.png'), fullPage: true });
  await page.evaluate(() => dispatchEvent(new Event('blur')));
  await expect(page.locator('#phase')).toHaveText('disconnected');
  await page.evaluate(() => dispatchEvent(new Event('focus')));
  await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
  expect(await page.evaluate(() => fake.calls.filter(x => x[0] === 'picker').length)).toBe(1);
});

test('missing Web Bluetooth is explained without claiming a device', async ({ page }) => {
  await page.unroute('**/app.js');
  await page.addInitScript(() => Object.defineProperty(navigator, 'bluetooth', { value: undefined }));
  await page.reload();
  await expect(page.locator('#message')).toContainText('Open this page in Bluefy');
  await page.getByRole('button', { name: 'Choose & connect' }).click();
  await expect(page.locator('#phase')).toHaveText('disconnected');
});
