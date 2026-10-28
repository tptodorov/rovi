import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';

async function open(page, options = {}) {
  await page.route('**/fake-ble.js', async route => route.fulfill({
    contentType: 'text/javascript', body: await readFile(new URL('../fake-ble.js', import.meta.url), 'utf8'),
  }));
  await page.route('**/app*.js*', async route => route.fulfill({ contentType: 'text/javascript', body: `
import { fakeBle } from './fake-ble.js';
const fake = fakeBle(${JSON.stringify(options)});
Object.defineProperty(navigator, 'bluetooth', { value: fake.bluetooth, configurable: true });
window.fake = fake;
${await readFile(new URL('../../app.js', import.meta.url), 'utf8')}
` }));
  await page.goto('/'); await page.locator('#connect').click();
  await expect(page.locator('body')).toHaveAttribute('data-phase', 'ready');
  await page.locator('#diagnostics summary').click();
}

test('network information updates and disconnect labels it last-reported', async ({ page }, info) => {
  await open(page);
  await expect(page.locator('#network-info')).toContainText('Home Wi-Fi');
  await expect(page.locator('#network-info')).toContainText('192.168.1.193');
  await page.evaluate(() => fake.notifyNetwork([0, 0, 5, 0, 0, 0, 0, 0]));
  await expect(page.locator('#network-info')).toContainText('Car network');
  await expect(page.locator('#network-info')).toContainText('Home address unavailable');
  await page.screenshot({ path: info.outputPath('network-controls.png'), fullPage: true });
  await page.locator('#disconnect').click();
  await expect(page.locator('#network-info')).toContainText('Last reported:');
  await expect(page.locator('#network-home')).toBeDisabled();
});

for (const [button, code] of [['network-home', 3], ['network-car', 4]]) {
  test(`${button} confirms and disconnects without ARM or nonzero input`, async ({ page }) => {
    await open(page); await page.locator('#'+button).click();
    await expect(page.locator('body')).toHaveAttribute('data-phase', 'disconnected');
    await expect(page.locator('#network-message')).toContainText('Reconnect to check');
    expect(await page.evaluate(code => fake.calls.some(([kind, value]) => kind === 'control' && value === code), code)).toBe(true);
    expect(await page.evaluate(() => fake.calls.some(([kind, value]) => kind === 'control' && value === 1))).toBe(false);
    expect(await page.evaluate(() => fake.calls.filter(([kind]) => kind === 'setpoint').every(([, data]) => {
      const v = new DataView(Uint8Array.from(data).buffer); return [24,32,64].every(i => v.getFloat64(i,true) === 0);
    }))).toBe(true);
  });
}

test('armed network controls are disabled and become available after STOP', async ({ page }) => {
  await open(page); await page.locator('#arm').click();
  await expect(page.locator('#local-arm')).toHaveText('Armed');
  await expect(page.locator('#network-home')).toBeDisabled();
  await expect(page.locator('#network-car')).toBeDisabled();
  await page.locator('#stop').click();
  await expect(page.locator('#network-home')).toBeEnabled();
  expect(await page.evaluate(() => fake.calls.some(([kind, code]) => kind === 'control' && [3,4].includes(code)))).toBe(false);
});

test('unconfigured Home explains refusal and stays connected', async ({ page }) => {
  await open(page, { homeConfigured: false });
  await page.locator('#network-home').click();
  await expect(page.locator('#network-message')).toContainText('unavailable');
  await page.waitForTimeout(600);
  await expect(page.locator('#network-message')).toContainText('unavailable');
  await expect(page.locator('body')).toHaveAttribute('data-phase', 'ready');
  await expect(page.locator('#local-arm')).toHaveText('Disarmed');
});

test('older firmware keeps ordinary controls with network actions unavailable', async ({ page }) => {
  await open(page, { network: false });
  await expect(page.locator('#network-info')).toHaveText('Network information unavailable');
  await expect(page.locator('#network-home')).toBeDisabled();
  await expect(page.locator('#network-car')).toBeDisabled();
  await expect(page.locator('#arm')).toBeEnabled();
});
