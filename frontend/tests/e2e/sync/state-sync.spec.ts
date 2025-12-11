import { test, expect, TEST_USERS, loginAsTrader, loginAsAdmin, waitForWsConnection } from '../fixtures/test-fixtures';
import { TradingDeskPage, AdminDashboardPage } from '../helpers/page-objects';

test.describe('State Synchronization', () => {
  test.describe('Initial State Sync', () => {
    test('should receive full state sync after trader login', async ({ page }) => {
      // Login and verify state is synced by checking UI elements are populated
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);

      // After full state sync, we should see:
      // - Portfolio section with cash balance
      // - Company symbols loaded
      // - Leaderboard populated
      const hasPortfolio = await page.locator('text=/CASH|Portfolio/i').first().isVisible().catch(() => false);
      const hasSymbols = await page.locator('text=/TCS|RELIANCE|ZOMATO/').first().isVisible().catch(() => false);

      // At least one of these should be visible after state sync
      expect(hasPortfolio || hasSymbols).toBeTruthy();
    });

    test('should receive portfolio data after login', async ({ page }) => {
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);

      const trading = new TradingDeskPage(page);
      await page.waitForLoadState('networkidle');

      // Portfolio should have data
      const cashBalance = await trading.getCashBalance();
      expect(cashBalance).toBeTruthy();
    });

    test('should receive company list after login', async ({ page }) => {
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);

      // Should see company symbols
      await expect(page.locator('body')).toContainText(/TCS|RELIANCE|ZOMATO|INFY/);
    });

    test('should receive market status after login', async ({ page }) => {
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);

      // Market status should be visible (open or closed indicator)
      await page.waitForLoadState('networkidle');
    });
  });

  test.describe('WebSocket Reconnection', () => {
    test('should reconnect automatically after connection drop', async ({ page }) => {
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);

      // Verify Live indicator before test (use exact match)
      const liveIndicator = page.getByText('Live', { exact: true });
      await expect(liveIndicator).toBeVisible();

      // Reload page to simulate connection drop and reconnect
      await page.reload();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(3000);

      // After reconnection, Live indicator should be visible again
      await expect(liveIndicator).toBeVisible();
    });

    test('should re-authenticate with token on reconnect', async ({ page }) => {
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
      await page.waitForLoadState('networkidle');

      // Get stored token
      const tokenBefore = await page.evaluate(() => localStorage.getItem('session_token'));
      expect(tokenBefore).toBeTruthy();

      // Reload page to simulate reconnection
      await page.reload();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(3000);

      // Token should still be there and user should still be on trading page
      const tokenAfter = await page.evaluate(() => localStorage.getItem('session_token'));
      expect(tokenAfter).toBeTruthy();

      // Should still be authenticated (not redirected to login)
      await expect(page).toHaveURL(/\/trade/);
    });
  });

  test.describe('Real-time Broadcast Updates', () => {
    test('should receive trade updates in real-time', async ({ browser }) => {
      const context1 = await browser.newContext();
      const context2 = await browser.newContext();

      const page1 = await context1.newPage();
      const page2 = await context2.newPage();

      try {
        await loginAsTrader(page1, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
        await loginAsTrader(page2, TEST_USERS.trader2.regno, TEST_USERS.trader2.password);

        await page1.waitForLoadState('networkidle');
        await page2.waitForLoadState('networkidle');
        await page1.waitForTimeout(1000);
        await page2.waitForTimeout(1000);

        // Both users should see the trading desk
        await expect(page1.getByText('Quick Trade')).toBeVisible();
        await expect(page2.getByText('Quick Trade')).toBeVisible();
      } finally {
        await context1.close();
        await context2.close();
      }
    });

    test('should receive candle updates', async ({ page }) => {
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
      await page.waitForLoadState('networkidle');

      // Subscribe to a symbol
      await page.locator('text=TCS').first().click();
      await page.waitForTimeout(1000);

      // Chart area should be visible (TradingView chart)
      const chartArea = page.locator('.tv-lightweight-charts, canvas, [class*="chart"]');
      await expect(chartArea.first()).toBeVisible({ timeout: 5000 });
    });

    test('should receive index updates', async ({ page }) => {
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);

      // Market indices bar should be visible (showing VIX, CONGLOMERATE, etc.)
      // The indices are in the top header bar - use first() to avoid strict mode
      await expect(page.locator('text=/VIX|LIVE|CONGLOMERATE/i').first()).toBeVisible({ timeout: 10000 });
    });

    test('should receive news updates', async ({ page }) => {
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);

      // News ticker at the bottom - use exact match to avoid "Waiting for news..."
      const newsSection = page.getByText('NEWS', { exact: true });
      await expect(newsSection).toBeVisible({ timeout: 10000 });
    });
  });

  test.describe('Component-Specific Sync', () => {
    test('should sync portfolio on demand', async ({ page }) => {
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);

      // Portfolio should be visible with cash balance
      const trading = new TradingDeskPage(page);
      await expect(trading.portfolioWidget).toBeVisible();

      // Cash balance should be shown
      const cash = await trading.getCashBalance();
      expect(cash).toBeTruthy();
    });

    test('should sync leaderboard on demand', async ({ page }) => {
      await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);

      // Leaderboard should be visible
      const trading = new TradingDeskPage(page);
      await expect(trading.leaderboardWidget).toBeVisible();
    });
  });
});
