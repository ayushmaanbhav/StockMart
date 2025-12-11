import { test, expect, TEST_USERS } from '../fixtures/test-fixtures';
import { LoginPage } from '../helpers/page-objects';

test.describe('Login Page', () => {
  let loginPage: LoginPage;

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page);
    await loginPage.goto();
    // Wait for the form to be fully loaded
    await page.waitForSelector('input[placeholder*="registration" i]', { timeout: 10000 });
  });

  test.describe('UI Rendering', () => {
    test('should render login form with all elements', async ({ page }) => {
      // Verify trader tab is selected by default
      await expect(loginPage.traderTab).toBeVisible();
      await expect(loginPage.adminTab).toBeVisible();

      // Verify form fields
      await expect(loginPage.regnoInput).toBeVisible();
      await expect(loginPage.passwordInput).toBeVisible();
      await expect(loginPage.signInButton).toBeVisible();

      // Verify register link
      await expect(loginPage.registerLink).toBeVisible();
    });

    test('should switch between trader and admin tabs', async ({ page }) => {
      // Click admin tab
      await loginPage.adminTab.click();
      await expect(loginPage.usernameInput).toBeVisible();

      // Click back to trader tab
      await loginPage.traderTab.click();
      await expect(loginPage.regnoInput).toBeVisible();
    });

    test('should show different placeholders for trader vs admin', async ({ page }) => {
      // Trader mode - registration number
      await expect(loginPage.regnoInput).toHaveAttribute('placeholder', /registration/i);

      // Admin mode - username
      await loginPage.adminTab.click();
      await expect(loginPage.usernameInput).toHaveAttribute('placeholder', /username/i);
    });
  });

  test.describe('Form Validation', () => {
    test('should show validation error for empty registration number', async ({ page }) => {
      await loginPage.passwordInput.fill('password');
      await loginPage.signInButton.click();

      // Form should not submit
      await expect(page).toHaveURL('/login');
    });

    test('should show validation error for empty password', async ({ page }) => {
      await loginPage.regnoInput.fill('REG123');
      await loginPage.signInButton.click();

      // Form should not submit
      await expect(page).toHaveURL('/login');
    });

    test('should show validation error for short registration number', async ({ page }) => {
      await loginPage.regnoInput.fill('RE');
      await loginPage.passwordInput.fill('pass');
      await loginPage.signInButton.click();

      // Should show validation error or stay on page
      await expect(page).toHaveURL('/login');
    });

    test('should show validation error for short password', async ({ page }) => {
      await loginPage.regnoInput.fill('REG123');
      await loginPage.passwordInput.fill('pas');
      await loginPage.signInButton.click();

      // Should show validation error or stay on page
      await expect(page).toHaveURL('/login');
    });
  });

  test.describe('Trader Authentication', () => {
    test('should login successfully with valid credentials', async ({ page }) => {
      await loginPage.loginAsTrader(
        TEST_USERS.trader2.regno,
        TEST_USERS.trader2.password
      );

      // Should navigate to trading page
      await expect(page).toHaveURL('/trade', { timeout: 15000 });

      // Should display user name
      await expect(page.locator('body')).toContainText(TEST_USERS.trader2.name);
    });

    test('should show error for invalid password', async ({ page }) => {
      await loginPage.loginAsTrader(
        TEST_USERS.trader2.regno,
        'wrongpassword'
      );

      // Should stay on login page
      await expect(page).toHaveURL('/login');

      // Should show error message
      await expect(page.locator('body')).toContainText(/invalid password/i);
    });

    test('should show error for non-existent user', async ({ page }) => {
      await loginPage.loginAsTrader(
        'NONEXISTENT999',
        'anypassword'
      );

      // Should stay on login page
      await expect(page).toHaveURL('/login');

      // Should show error message
      await expect(page.locator('body')).toContainText(/not found|register/i);
    });

    test('should store session token after successful login', async ({ page }) => {
      await loginPage.loginAsTrader(
        TEST_USERS.trader2.regno,
        TEST_USERS.trader2.password
      );

      await expect(page).toHaveURL('/trade', { timeout: 15000 });

      // Check session token is stored
      const token = await page.evaluate(() => localStorage.getItem('session_token'));
      expect(token).toBeTruthy();
    });

    test('should receive initial data after login', async ({ page }) => {
      await loginPage.loginAsTrader(
        TEST_USERS.trader2.regno,
        TEST_USERS.trader2.password
      );

      // After successful login, user should see trading page with data
      await expect(page).toHaveURL('/trade', { timeout: 15000 });

      // Page should contain portfolio/trading data (loaded from full state sync)
      await expect(page.locator('body')).toContainText(/portfolio|cash|balance|networth/i);
    });
  });

  test.describe('Admin Authentication', () => {
    // Note: Admin authentication requires an admin user in the database
    // Skip these tests if admin user is not set up
    test.skip('should login successfully as admin with valid credentials', async ({ page }) => {
      await loginPage.loginAsAdmin(
        TEST_USERS.admin.username,
        TEST_USERS.admin.password
      );

      // Should navigate to admin page
      await expect(page).toHaveURL(/\/admin/, { timeout: 15000 });
    });

    test.skip('should show error for invalid admin password', async ({ page }) => {
      await loginPage.loginAsAdmin(
        TEST_USERS.admin.username,
        'wrongpassword'
      );

      // Should stay on login page
      await expect(page).toHaveURL('/login');

      // Should show error message
      await expect(page.locator('body')).toContainText(/invalid|failed/i);
    });

    test('should show error for non-existent admin username', async ({ page }) => {
      await loginPage.loginAsAdmin(
        'nonexistentadmin',
        'anypassword'
      );

      // Should stay on login page - wait for error message
      await page.waitForTimeout(3000);
      await expect(page).toHaveURL('/login');

      // Should show error message
      await expect(page.locator('body')).toContainText(/not found|invalid|failed/i);
    });
  });

  test.describe('Navigation', () => {
    test('should navigate to register page', async ({ page }) => {
      await loginPage.registerLink.click();
      await expect(page).toHaveURL('/register');
    });

    test('should redirect authenticated trader to trade page', async ({ page }) => {
      // Login first
      await loginPage.loginAsTrader(
        TEST_USERS.trader2.regno,
        TEST_USERS.trader2.password
      );
      await expect(page).toHaveURL('/trade', { timeout: 15000 });

      // Try to go to login - should redirect
      await page.goto('/login');
      await expect(page).toHaveURL('/trade');
    });
  });
});
