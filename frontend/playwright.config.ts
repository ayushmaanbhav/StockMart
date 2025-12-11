import { defineConfig, devices } from '@playwright/test';

/**
 * Comprehensive Playwright E2E Test Configuration
 *
 * Optimized for:
 * - 62GB RAM, 16 cores - full parallel execution
 * - Multi-user trading scenarios
 * - Real-time WebSocket state validation
 * - 95%+ code coverage target
 */
export default defineConfig({
  testDir: './tests/e2e',

  // Run tests in parallel with all available workers
  fullyParallel: true,
  workers: 8, // Use 8 workers for 16 cores (leave room for browsers)

  // Fail the build on CI if you accidentally left test.only in the source code
  forbidOnly: !!process.env.CI,

  // Retry failed tests up to 2 times (helps with flaky WebSocket tests)
  retries: 2,

  // Reporter configuration
  reporter: [
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
    ['json', { outputFile: 'playwright-report/results.json' }],
    ['list'],
  ],

  // Shared settings for all projects
  use: {
    // Base URL for the application
    baseURL: 'http://localhost:5174',

    // Collect trace on retry (helps debug flaky tests)
    trace: 'on-first-retry',

    // Screenshot on failure
    screenshot: 'only-on-failure',

    // Video recording
    video: 'retain-on-failure',

    // Relaxed navigation timeout for WebSocket connections
    navigationTimeout: 45000,

    // Relaxed action timeout
    actionTimeout: 15000,

    // Viewport
    viewport: { width: 1920, height: 1080 },
  },

  // Test timeout - increased for complex multi-step tests
  timeout: 90000,

  // Expect timeout - increased for WebSocket state updates
  expect: {
    timeout: 15000,
  },

  // Projects configuration
  projects: [
    // Setup project - runs first to prepare test data
    {
      name: 'setup',
      testMatch: /global\.setup\.ts/,
    },

    // Chromium tests (main browser) - parallel execution
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1920, height: 1080 },
      },
      dependencies: ['setup'],
      // Exclude multi-user and state-sync tests (they run in separate projects)
      testIgnore: [/multi-user.*\.spec\.ts/, /state-sync.*\.spec\.ts/],
    },

    // Multi-user tests - run serially with longer timeouts
    {
      name: 'multi-user',
      testMatch: /multi-user.*\.spec\.ts/,
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1280, height: 720 },
        // Longer timeouts for multi-user scenarios
        actionTimeout: 20000,
        navigationTimeout: 60000,
      },
      dependencies: ['setup'],
      // Run multi-user tests serially
      fullyParallel: false,
      // Extra timeout for multi-browser coordination
      timeout: 120000,
    },

    // State sync tests - run serially to avoid WebSocket conflicts
    {
      name: 'state-sync',
      testMatch: /state-sync.*\.spec\.ts/,
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1920, height: 1080 },
        // Longer timeouts for WebSocket sync
        actionTimeout: 20000,
      },
      dependencies: ['setup'],
      // Run state sync tests serially
      fullyParallel: false,
      timeout: 120000,
    },
  ],

  // Web server configuration - start frontend if not running
  webServer: [
    {
      command: 'npm run dev',
      url: 'http://localhost:5174',
      reuseExistingServer: true,
      timeout: 60000,
    },
  ],
});
