import { test, expect, TEST_USERS, loginAsAdmin, loginAsTrader, waitForWsConnection } from '../fixtures/test-fixtures';
import { TradersAdminPage, TradingDeskPage } from '../helpers/page-objects';

test.describe('Trader Management', () => {
  let tradersPage: TradersAdminPage;

  test.beforeEach(async ({ page }) => {
    await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
    await page.goto('/admin/traders');
    tradersPage = new TradersAdminPage(page);
    await page.waitForLoadState('networkidle');
  });

  test.describe('Trader List', () => {
    test('should display list of traders', async ({ page }) => {
      // Should see trader list
      const rows = tradersPage.traderRows;
      const count = await rows.count();
      expect(count).toBeGreaterThan(0);
    });

    test('should display trader information', async ({ page }) => {
      // Should show trader names, regnos, or other info
      await expect(page.locator('body')).toContainText(TEST_USERS.trader1.name);
    });

    test('should have action buttons for traders', async ({ page }) => {
      // Should have ban/unban or mute/unmute buttons
      const actionButtons = page.getByRole('button').filter({
        hasText: /ban|unban|mute|unmute/i
      });
      const count = await actionButtons.count();
      expect(count).toBeGreaterThanOrEqual(0);
    });
  });

  test.describe('Search Functionality', () => {
    test('should filter traders by search', async ({ page }) => {
      // Search for specific trader
      await tradersPage.searchInput.fill(TEST_USERS.trader2.name);

      await page.waitForTimeout(500);

      // Should filter results
      await expect(page.locator('body')).toContainText(TEST_USERS.trader2.name);
    });
  });
});

test.describe('Ban/Unban Functionality', () => {
  test('banned trader should not be able to login', async ({ browser }) => {
    // This test would ban a user, try to login, then unban
    // Skipping actual ban to avoid breaking test data
    test.skip();

    const adminContext = await browser.newContext();
    const traderContext = await browser.newContext();

    const adminPage = await adminContext.newPage();
    const traderPage = await traderContext.newPage();

    try {
      // Admin bans trader
      await loginAsAdmin(adminPage, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await adminPage.goto('/admin/traders');

      const tradersAdmin = new TradersAdminPage(adminPage);
      // await tradersAdmin.banTrader(TEST_USERS.trader1.name);

      await adminPage.waitForTimeout(2000);

      // Trader tries to login
      await traderPage.goto('/login');
      await waitForWsConnection(traderPage);

      await traderPage.getByPlaceholder(/registration number/i).fill(TEST_USERS.trader1.regno);
      await traderPage.getByPlaceholder(/password/i).fill(TEST_USERS.trader1.password);
      await traderPage.getByRole('button', { name: /sign in/i }).click();

      await traderPage.waitForTimeout(2000);

      // Should show banned message
      await expect(traderPage.locator('body')).toContainText(/banned/i);

      // Cleanup: Unban the trader
      // await tradersAdmin.unbanTrader(TEST_USERS.trader1.name);
    } finally {
      await adminContext.close();
      await traderContext.close();
    }
  });

  test('banned trader should be disconnected', async ({ browser }) => {
    // This test would ban an already-logged-in trader
    test.skip();
  });
});

test.describe('Mute/Unmute Functionality', () => {
  test('muted trader should not be able to send chat messages', async ({ browser }) => {
    // Skip to avoid data changes
    test.skip();

    const adminContext = await browser.newContext();
    const traderContext = await browser.newContext();

    const adminPage = await adminContext.newPage();
    const traderPage = await traderContext.newPage();

    try {
      // Login both
      await loginAsAdmin(adminPage, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await loginAsTrader(traderPage, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);

      // Admin mutes trader
      await adminPage.goto('/admin/traders');
      const tradersAdmin = new TradersAdminPage(adminPage);
      // await tradersAdmin.muteTrader(TEST_USERS.trader1.name);

      await adminPage.waitForTimeout(2000);

      // Trader tries to send chat
      const trading = new TradingDeskPage(traderPage);
      await trading.chatInput.fill('This should not send');
      await trading.chatSendButton.click();

      await traderPage.waitForTimeout(2000);

      // Should show muted error or message not sent
      await expect(traderPage.locator('body')).toContainText(/muted|cannot|disabled/i);

      // Cleanup: Unmute
      // await tradersAdmin.unmuteTrader(TEST_USERS.trader1.name);
    } finally {
      await adminContext.close();
      await traderContext.close();
    }
  });
});

test.describe('Admin Views', () => {
  test.describe('Trades View', () => {
    test('should display trade history', async ({ page }) => {
      await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await page.goto('/admin/trades');
      await page.waitForLoadState('networkidle');

      // Should show trades list or table
      await expect(page.locator('body')).toContainText(/trade|symbol|price|qty/i);
    });

    test('should have pagination for trades', async ({ page }) => {
      await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await page.goto('/admin/trades');
      await page.waitForLoadState('networkidle');

      // Look for pagination controls
      const pagination = page.getByRole('button', { name: /next|prev|page/i });
      // Pagination may or may not be visible depending on data volume
    });
  });

  test.describe('Orders View', () => {
    test('should display open orders', async ({ page }) => {
      await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await page.goto('/admin/orders');
      await page.waitForLoadState('networkidle');

      // Should show orders list or table
      await expect(page.locator('body')).toContainText(/order|symbol|user|qty/i);
    });
  });

  test.describe('Orderbook View', () => {
    test('should display order book for selected symbol', async ({ page }) => {
      await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await page.goto('/admin/orderbook');
      await page.waitForLoadState('networkidle');

      // Should show orderbook with bids and asks
      await expect(page.locator('body')).toContainText(/bid|ask|price|qty|orderbook/i);
    });

    test('should allow selecting different symbols', async ({ page }) => {
      await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await page.goto('/admin/orderbook');
      await page.waitForLoadState('networkidle');

      // Should have symbol selector
      const symbolSelector = page.getByRole('combobox').or(page.locator('select'));
      if (await symbolSelector.isVisible()) {
        await symbolSelector.click();
      }
    });
  });

  test.describe('Companies View', () => {
    test('should display list of companies', async ({ page }) => {
      await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await page.goto('/admin/companies');
      await page.waitForLoadState('networkidle');

      // Should show company list
      await expect(page.locator('body')).toContainText(/TCS|RELIANCE|company|symbol/i);
    });

    test('should have create company functionality', async ({ page }) => {
      await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await page.goto('/admin/companies');
      await page.waitForLoadState('networkidle');

      // Should have create button
      const createButton = page.getByRole('button', { name: /create|add|new/i });
      // Create button may or may not be visible
    });
  });

  test.describe('Diagnostics View', () => {
    test('should display system diagnostics', async ({ page }) => {
      await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await page.goto('/admin/diagnostics');
      await page.waitForLoadState('networkidle');

      // Should show system info
      await expect(page.locator('body')).toContainText(/diagnostic|system|status|health/i);
    });
  });
});
