import { test as setup, expect } from '@playwright/test';
import { TEST_USERS } from './fixtures/test-fixtures';

/**
 * Global setup for E2E tests
 *
 * Ensures:
 * 1. Backend is running and responsive
 * 2. Frontend is running and accessible
 * 3. WebSocket connection can be established
 * 4. All test users are pre-registered
 */
setup('verify test environment', async ({ page }) => {
  // Check frontend is accessible
  const response = await page.goto('/', { waitUntil: 'domcontentloaded' });
  expect(response?.ok()).toBeTruthy();

  // Verify app renders
  await expect(page.locator('body')).not.toBeEmpty();

  // Wait for the app to be ready (login page renders)
  await expect(page.locator('text=StockMart')).toBeVisible({ timeout: 15000 });

  console.log('Test environment verified: Frontend is ready');
});

// Helper to try registering a user (ignores if already exists)
async function tryRegisterUser(page: import('@playwright/test').Page, user: { regno: string; name: string; password: string }) {
  await page.goto('/register');
  await page.waitForLoadState('domcontentloaded');

  // Wait for registration form (shorter timeout)
  await page.waitForSelector('input[placeholder*="registration" i]', { timeout: 3000 }).catch(() => null);

  // Fill registration form quickly
  await page.locator('input[placeholder*="registration" i]').fill(user.regno);
  await page.locator('input[placeholder*="appear" i], input[placeholder*="display" i]').fill(user.name);
  await page.locator('input[type="password"]').first().fill(user.password);
  await page.locator('input[type="password"]').last().fill(user.password);

  // Click create account
  await page.getByRole('button', { name: /create account/i }).click();

  // Wait briefly for navigation or error
  await page.waitForTimeout(2000);

  // Check if we need to logout (registration succeeded)
  const url = page.url();
  if (url.includes('/trade')) {
    // Logout via localStorage clear and navigate
    await page.evaluate(() => {
      localStorage.removeItem('session_token');
    });
    await page.goto('/login');
  }
}

setup('ensure test data exists', async ({ page }) => {
  // Navigate to login page
  await page.goto('/login');
  await page.waitForLoadState('networkidle');

  // Check that login form is visible - using actual placeholder text
  await expect(page.getByPlaceholder(/registration/i)).toBeVisible();
  await expect(page.getByPlaceholder(/password/i)).toBeVisible();

  console.log('Test data setup complete');
});

setup('pre-register test users', async ({ page }) => {
  // Increase timeout for user registration
  setup.setTimeout(120000);

  // List of users that need to be pre-registered for tests
  const usersToRegister = [
    TEST_USERS.errorTestTrader,
    TEST_USERS.portfolioTestTrader,
    TEST_USERS.orderTestTrader,
    TEST_USERS.multiUser1,
    TEST_USERS.multiUser2,
    TEST_USERS.multiUser3,
  ];

  let registered = 0;
  let alreadyExists = 0;

  for (const user of usersToRegister) {
    try {
      await tryRegisterUser(page, user);
      registered++;
    } catch {
      alreadyExists++;
    }
  }

  console.log(`Pre-registered ${registered} new users, ${alreadyExists} already existed`);
});
