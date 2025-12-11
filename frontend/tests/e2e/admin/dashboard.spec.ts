import { test, expect, TEST_USERS, loginAsAdmin, waitForWsConnection } from '../fixtures/test-fixtures';
import { AdminDashboardPage } from '../helpers/page-objects';

test.describe('Admin Dashboard', () => {
  let dashboardPage: AdminDashboardPage;

  test.beforeEach(async ({ page }) => {
    await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
    dashboardPage = new AdminDashboardPage(page);
    await page.waitForLoadState('networkidle');
  });

  test.describe('Dashboard UI', () => {
    test('should display admin navigation sidebar', async ({ page }) => {
      await expect(dashboardPage.dashboardLink).toBeVisible();
      await expect(dashboardPage.gameControlLink).toBeVisible();
      await expect(dashboardPage.tradersLink).toBeVisible();
      await expect(dashboardPage.companiesLink).toBeVisible();
      await expect(dashboardPage.tradesLink).toBeVisible();
      await expect(dashboardPage.ordersLink).toBeVisible();
      await expect(dashboardPage.orderbookLink).toBeVisible();
    });

    test('should display dashboard metrics', async ({ page }) => {
      // Dashboard should show various metrics
      await expect(page.locator('body')).toContainText(/active|traders|trades|volume|orders/i);
    });

    test('should have logout button', async ({ page }) => {
      await expect(dashboardPage.logoutButton).toBeVisible();
    });
  });

  test.describe('Navigation', () => {
    test('should navigate to Game Control page', async ({ page }) => {
      await dashboardPage.navigateToGameControl();
      await expect(page).toHaveURL(/\/admin\/game/);
    });

    test('should navigate to Traders page', async ({ page }) => {
      await dashboardPage.navigateToTraders();
      await expect(page).toHaveURL(/\/admin\/traders/);
    });

    test('should navigate to Companies page', async ({ page }) => {
      await dashboardPage.navigateToCompanies();
      await expect(page).toHaveURL(/\/admin\/companies/);
    });

    test('should navigate to Trades page', async ({ page }) => {
      await dashboardPage.navigateToTrades();
      await expect(page).toHaveURL(/\/admin\/trades/);
    });

    test('should navigate to Orders page', async ({ page }) => {
      await dashboardPage.navigateToOrders();
      await expect(page).toHaveURL(/\/admin\/orders/);
    });

    test('should navigate to Orderbook page', async ({ page }) => {
      await dashboardPage.navigateToOrderbook();
      await expect(page).toHaveURL(/\/admin\/orderbook/);
    });

    test('should navigate to Diagnostics page', async ({ page }) => {
      await dashboardPage.navigateToDiagnostics();
      await expect(page).toHaveURL(/\/admin\/diagnostics/);
    });
  });

  test.describe('Logout', () => {
    test('should logout admin and redirect to login', async ({ page }) => {
      await dashboardPage.logout();
      await expect(page).toHaveURL('/login');

      // Session token should be cleared
      const token = await page.evaluate(() => localStorage.getItem('session_token'));
      expect(token).toBeNull();
    });
  });
});
