import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';

test.beforeEach(async ({ page }) => {
  await page.route('**/app*.js*', async route => route.fulfill({
    contentType: 'text/javascript', body: await readFile(new URL('../ui-session.js', import.meta.url), 'utf8'),
  }));
  await page.goto('/');
});

test('hold/release and sliding outside cannot replay a direction', async ({ page }) => {
  await page.getByRole('button', { name: 'ARM AT ZERO' }).click();
  const forward = page.getByRole('button', { name: 'Hold to move forward', exact: true });
  const box = await forward.boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await expect(page.locator('#requested')).toContainText('vx 0.05');
  await page.mouse.move(1, 1);
  await expect(page.locator('#requested')).toContainText('vx 0.00');
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await expect(page.locator('#requested')).toContainText('vx 0.00');
  await page.mouse.up();
  await page.mouse.down();
  await expect(page.locator('#requested')).toContainText('vx 0.05');
  await page.mouse.up();
  await expect(page.locator('#requested')).toContainText('vx 0.00');
});

test('click activation cannot latch motion; STOP disables directions until fresh ARM', async ({ page }) => {
  await page.getByRole('button', { name: 'ARM AT ZERO' }).click();
  await page.getByRole('button', { name: 'Hold to strafe left', exact: true }).evaluate(button => button.click());
  await expect(page.locator('#requested')).toContainText('vy 0.00');
  await page.getByRole('button', { name: 'STOP', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Hold to strafe left', exact: true })).toBeDisabled();
  await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
});

test('zero axes disabled, layout fits a phone and larger text, no external assets', async ({ page }) => {
  await page.getByRole('button', { name: 'ARM AT ZERO' }).click();
  await page.evaluate(() => { fixture.caps.max_vy = 0; document.documentElement.style.fontSize = '24px'; });
  await expect(page.getByRole('button', { name: 'Hold to strafe left', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  for (const button of await page.locator('.direction:visible').all()) {
    const box = await button.boundingBox(); expect(box.width).toBeGreaterThanOrEqual(44); expect(box.height).toBeGreaterThanOrEqual(44);
  }
  const assets = await page.evaluate(() => performance.getEntriesByType('resource').map(x => x.name));
  expect(assets.every(url => new URL(url).origin === 'https://127.0.0.1:8177')).toBe(true);
});

test('blur ends the session; a foreground return cannot re-arm it', async ({ page }) => {
  await page.getByRole('button', { name: 'ARM AT ZERO' }).click();
  await page.evaluate(() => dispatchEvent(new Event('blur')));
  await expect(page.locator('#phase')).toHaveText('disconnected');
  await page.evaluate(() => dispatchEvent(new Event('focus')));
  await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
});

test('hidden page or pagehide ends the armed session', async ({ page }) => {
  for (const event of ['visibilitychange', 'pagehide']) {
    await page.evaluate(() => { fixture.phase = 'ready'; fixture.transport.connected = true; fixture.changed(); });
    await page.getByRole('button', { name: 'ARM AT ZERO' }).click();
    await expect(page.locator('#local-arm')).toHaveText('Locally armed');
    await page.evaluate(name => {
      if (name === 'visibilitychange') Object.defineProperty(document, 'hidden', { value: true, configurable: true });
      (name === 'pagehide' ? window : document).dispatchEvent(new Event(name));
      if (name === 'visibilitychange') delete document.hidden;
    }, event);
    await expect(page.locator('#phase')).toHaveText('disconnected');
    await expect(page.locator('#local-arm')).toHaveText('Locally disarmed');
  }
});
