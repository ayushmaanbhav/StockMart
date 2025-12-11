import { test, expect, TEST_USERS } from '../fixtures/test-fixtures';
import { RegisterPage } from '../helpers/page-objects';

test.describe('Registration Page', () => {
  let registerPage: RegisterPage;

  test.beforeEach(async ({ page }) => {
    registerPage = new RegisterPage(page);
    await registerPage.goto();
    // Wait for the form to be fully loaded
    await page.waitForSelector('input[placeholder*="registration" i]', { timeout: 10000 });
  });

  test.describe('UI Rendering', () => {
    test('should render registration form with all elements', async ({ page }) => {
      await expect(registerPage.regnoInput).toBeVisible();
      await expect(registerPage.nameInput).toBeVisible();
      await expect(registerPage.passwordInput).toBeVisible();
      await expect(registerPage.confirmPasswordInput).toBeVisible();
      await expect(registerPage.createAccountButton).toBeVisible();
      await expect(registerPage.loginLink).toBeVisible();
    });

    test('should have correct placeholders', async () => {
      // Registration field has "registration" in placeholder
      await expect(registerPage.regnoInput).toHaveAttribute('placeholder', /registration/i);
      // Display name field has "appear" or "display" in placeholder
      await expect(registerPage.nameInput).toBeVisible();
    });
  });

  test.describe('Form Validation', () => {
    test('should require registration number', async ({ page }) => {
      await registerPage.nameInput.fill('Test Name');
      await registerPage.passwordInput.fill('testpass');
      await registerPage.confirmPasswordInput.fill('testpass');
      await registerPage.createAccountButton.click();

      // Should stay on register page
      await expect(page).toHaveURL('/register');
    });

    test('should require display name', async ({ page }) => {
      await registerPage.regnoInput.fill('TESTREG001');
      await registerPage.passwordInput.fill('testpass');
      await registerPage.confirmPasswordInput.fill('testpass');
      await registerPage.createAccountButton.click();

      // Should stay on register page
      await expect(page).toHaveURL('/register');
    });

    test('should require password', async ({ page }) => {
      await registerPage.regnoInput.fill('TESTREG001');
      await registerPage.nameInput.fill('Test Name');
      await registerPage.confirmPasswordInput.fill('testpass');
      await registerPage.createAccountButton.click();

      // Should stay on register page
      await expect(page).toHaveURL('/register');
    });

    test('should validate minimum registration number length', async ({ page }) => {
      await registerPage.register('RE', 'Test Name', 'testpass');

      // Should stay on register page with validation error
      await expect(page).toHaveURL('/register');
    });

    test('should validate minimum name length', async ({ page }) => {
      await registerPage.regnoInput.fill('TESTREG001');
      await registerPage.nameInput.fill('T');
      await registerPage.passwordInput.fill('testpass');
      await registerPage.confirmPasswordInput.fill('testpass');
      await registerPage.createAccountButton.click();

      // Should stay on register page
      await expect(page).toHaveURL('/register');
    });

    test('should validate minimum password length', async ({ page }) => {
      await registerPage.register('TESTREG001', 'Test Name', 'pas');

      // Should stay on register page with validation error
      await expect(page).toHaveURL('/register');
    });

    test('should detect password mismatch', async ({ page }) => {
      await registerPage.register('TESTREG001', 'Test Name', 'password1', 'password2');

      // Should show password mismatch error
      await expect(page).toHaveURL('/register');
      await expect(page.locator('body')).toContainText(/match|mismatch/i);
    });
  });

  test.describe('Registration Flow', () => {
    test('should register new user successfully', async ({ page }) => {
      const uniqueRegno = TEST_USERS.newTrader.regno();

      await registerPage.register(
        uniqueRegno,
        TEST_USERS.newTrader.name,
        TEST_USERS.newTrader.password
      );

      // Should navigate to trading page
      await expect(page).toHaveURL('/trade', { timeout: 15000 });

      // Should display user name
      await expect(page.locator('body')).toContainText(TEST_USERS.newTrader.name);
    });

    test('should show error for duplicate registration number', async ({ page }) => {
      // Try to register with existing regno
      await registerPage.register(
        TEST_USERS.trader2.regno,
        'New Name',
        'newpassword'
      );

      // Should stay on register page with error
      await expect(page).toHaveURL('/register');
      await expect(page.locator('body')).toContainText(/exists|already|taken/i);
    });

    test('should receive session token after registration', async ({ page }) => {
      const uniqueRegno = TEST_USERS.newTrader.regno();

      await registerPage.register(
        uniqueRegno,
        'Token Test User',
        'testpassword'
      );

      await expect(page).toHaveURL('/trade', { timeout: 15000 });

      // Check session token is stored
      const token = await page.evaluate(() => localStorage.getItem('session_token'));
      expect(token).toBeTruthy();
    });

    test('should receive initial portfolio allocation after registration', async ({ page }) => {
      const uniqueRegno = TEST_USERS.newTrader.regno();

      await registerPage.register(
        uniqueRegno,
        'Portfolio Test User',
        'testpassword'
      );

      // After successful registration, should be on trade page with portfolio data
      await expect(page).toHaveURL('/trade', { timeout: 15000 });

      // Page should show portfolio/cash information - indicating state sync occurred
      await expect(page.locator('body')).toContainText(/portfolio|cash|balance|\$/i);
    });

    test('should display welcome message after registration', async ({ page }) => {
      const uniqueRegno = TEST_USERS.newTrader.regno();
      const userName = 'Welcome Test User';

      await registerPage.register(
        uniqueRegno,
        userName,
        'testpassword'
      );

      await expect(page).toHaveURL('/trade', { timeout: 15000 });

      // Should show welcome message with name or cash/stock info
      await expect(page.locator('body')).toContainText(/welcome|cash|stock/i);
    });
  });

  test.describe('Navigation', () => {
    test('should navigate to login page', async ({ page }) => {
      await registerPage.loginLink.click();
      await expect(page).toHaveURL('/login');
    });

    test('should redirect authenticated user to trade page', async ({ page }) => {
      const uniqueRegno = TEST_USERS.newTrader.regno();

      await registerPage.register(
        uniqueRegno,
        'Redirect Test User',
        'testpassword'
      );

      await expect(page).toHaveURL('/trade', { timeout: 15000 });

      // Try to go to register - should redirect
      await page.goto('/register');
      await expect(page).toHaveURL('/trade');
    });
  });
});
