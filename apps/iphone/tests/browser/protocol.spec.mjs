import { test, expect } from '@playwright/test';

// Spec *Same results on every binding*: no protocol module, no connection, no fallback codec.
for (const missing of ['rovi_protocol_bg', 'rovi_protocol']) {
  test(`without ${missing} Connect stays disabled and the page explains why`, async ({ page }) => {
    await page.route(new RegExp(`/${missing}(\\.[^/]+)?\\.(wasm|js)$`), route => route.abort());
    await page.goto('/');
    await expect(page.locator('#connect')).toBeDisabled();
    await expect(page.locator('#message')).toContainText('Car protocol module could not load');
  });
}

test('with the module loaded the page does not report a protocol problem', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('#message')).not.toContainText('Car protocol module');
});
