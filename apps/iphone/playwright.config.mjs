import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests/browser',
  workers: 1,
  forbidOnly: Boolean(process.env.CI),
  reporter: 'list',
  use: {
    baseURL: 'https://127.0.0.1:8177',
    ignoreHTTPSErrors: true,
    viewport: { width: 390, height: 844 },
    launchOptions: process.env.ROVI_CHROMIUM ? { executablePath: process.env.ROVI_CHROMIUM } : {},
  },
  webServer: {
    command: 'node tests/server.mjs',
    url: 'https://127.0.0.1:8177',
    ignoreHTTPSErrors: true,
    reuseExistingServer: false,
  },
});
