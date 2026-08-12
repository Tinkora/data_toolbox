import { defineConfig, devices } from '@playwright/test';

const viewports = [
  ['chromium-375', { width: 375, height: 812 }],
  ['chromium-768', { width: 768, height: 900 }],
  ['chromium-1024', { width: 1024, height: 900 }],
  ['chromium-1440', { width: 1440, height: 1000 }],
];

export default defineConfig({
  testDir: './tests/browser',
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 2 : undefined,
  reporter: process.env.CI ? 'github' : 'line',
  use: {
    baseURL: 'http://127.0.0.1:43173',
    trace: 'retain-on-failure',
    ...devices['Desktop Chrome'],
  },
  projects: viewports.map(([name, viewport]) => ({ name, use: { viewport } })),
  webServer: {
    command: 'python3 -m http.server 43173 --bind 127.0.0.1',
    url: 'http://127.0.0.1:43173/web/',
    reuseExistingServer: false,
  },
});
