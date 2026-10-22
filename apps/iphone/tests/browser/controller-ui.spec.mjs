import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';

test.beforeEach(async ({ page }) => {
  await page.route('**/app*.js*', async route => route.fulfill({
    contentType: 'text/javascript', body: await readFile(new URL('../ui-session.js', import.meta.url), 'utf8'),
  }));
  await page.goto('/');
});

for (const [width, height] of [[320, 568], [375, 667], [390, 844]]) {
  test(`all directions, requested motion and STOP fit ${width}×${height}`, async ({ page }, info) => {
    await page.setViewportSize({ width, height });
    await page.locator('#arm').click();
    for (const target of [...await page.locator('[data-direction]').all(), page.locator('#stop'), page.locator('#motion-values')]) {
      const box = await target.boundingBox();
      expect(box.y).toBeGreaterThanOrEqual(0);
      expect(box.y + box.height).toBeLessThanOrEqual(height);
    }
    const dock = await page.locator('.control-dock').boundingBox();
    for (const button of await page.locator('[data-direction]').all()) {
      const box = await button.boundingBox();
      expect(box.y + box.height).toBeLessThanOrEqual(dock.y);
      expect(box.width).toBeGreaterThanOrEqual(44); expect(box.height).toBeGreaterThanOrEqual(44);
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: info.outputPath(`buttons-${width}.png`) });
  });
}

test('native mode segments support keyboard navigation and clear arm', async ({ page }) => {
  await page.locator('#arm').click();
  await page.getByRole('radio', { name: 'Buttons', exact: true }).focus();
  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('radio', { name: 'Physical', exact: true })).toBeChecked();
  await expect(page.locator('#local-arm')).toHaveText('Disarmed');
  await page.keyboard.press('ArrowLeft');
  await expect(page.getByRole('radio', { name: 'Buttons', exact: true })).toBeChecked();
  const outline = await page.locator('label[for="mode-buttons"]').evaluate(node => getComputedStyle(node).outlineStyle);
  expect(outline).toBe('solid');
});

test('dark theme and 200% text preserve usable controls without horizontal overflow', async ({ page }, info) => {
  await page.emulateMedia({ colorScheme: 'dark', reducedMotion: 'reduce' });
  await page.setViewportSize({ width: 375, height: 667 });
  await page.locator('#arm').click();
  await page.evaluate(() => { document.documentElement.style.fontSize = '32px'; });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  const color = await page.locator('meta[name="theme-color"]').getAttribute('content');
  expect(color).toBe('#0f1a15');
  for (const button of await page.locator('[data-direction]').all()) {
    await button.scrollIntoViewIfNeeded(); await button.focus();
    const box = await button.boundingBox(), dock = await page.locator('.control-dock').boundingBox();
    const status = await page.locator('.state-panel').boundingBox();
    expect(box.width).toBeGreaterThanOrEqual(44); expect(box.height).toBeGreaterThanOrEqual(44);
    expect(box.y).toBeGreaterThanOrEqual(status.y + status.height);
    expect(box.y + box.height).toBeLessThanOrEqual(dock.y);
  }
  await page.screenshot({ path: info.outputPath('dark-large-text.png') });
});

test('landscape retains reachable STOP and no horizontal overflow', async ({ page }, info) => {
  await page.setViewportSize({ width: 844, height: 390 });
  await page.locator('#arm').click();
  await expect(page.locator('#stop')).toBeInViewport();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: info.outputPath('landscape.png') });
});

test('enabled controls and state text meet 4.5:1 contrast in both palettes', async ({ page }) => {
  await page.locator('#arm').click();
  for (const colorScheme of ['light', 'dark']) {
    await page.emulateMedia({ colorScheme });
    const ratios = await page.evaluate(() => {
      const rgb = color => color.match(/[\d.]+/g).map(Number);
      const luminance = color => rgb(color).slice(0, 3).map(value => {
        const s = value / 255; return s <= .04045 ? s / 12.92 : ((s + .055) / 1.055) ** 2.4;
      }).reduce((sum, value, index) => sum + value * [.2126, .7152, .0722][index], 0);
      return ['#stop', '[data-direction="forward"]', '#local-arm', '#value-vx', 'label[for="mode-buttons"]'].map(selector => {
        const node = document.querySelector(selector); let parent = node;
        while (parent.parentElement && rgb(getComputedStyle(parent).backgroundColor)[3] === 0) parent = parent.parentElement;
        const a = luminance(getComputedStyle(node).color), b = luminance(getComputedStyle(parent).backgroundColor);
        return { selector, ratio: (Math.max(a, b) + .05) / (Math.min(a, b) + .05) };
      });
    });
    for (const { selector, ratio } of ratios) expect(ratio, `${colorScheme} ${selector}`).toBeGreaterThanOrEqual(4.5);
  }
});
