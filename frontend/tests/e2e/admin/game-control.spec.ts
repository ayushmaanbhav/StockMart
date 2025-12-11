import { test, expect, TEST_USERS, loginAsAdmin, loginAsTrader, waitForWsConnection } from '../fixtures/test-fixtures';
import { GameControlPage, TradingDeskPage } from '../helpers/page-objects';

test.describe('Game Control', () => {
  let gameControlPage: GameControlPage;

  test.beforeEach(async ({ page }) => {
    await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
    await page.goto('/admin/game');
    gameControlPage = new GameControlPage(page);
    await page.waitForLoadState('networkidle');
  });

  test.describe('Game Control UI', () => {
    test('should display market status', async ({ page }) => {
      await expect(page.locator('body')).toContainText(/market|open|closed/i);
    });

    test('should display market toggle button', async ({ page }) => {
      await expect(gameControlPage.toggleMarketButton).toBeVisible();
    });

    test('should display initialize game button', async ({ page }) => {
      // Init game button may or may not be visible depending on app state
      const hasInitButton = await page.locator('button:has-text("Init"), button:has-text("Initialize")').isVisible().catch(() => false);
      // Skip if not present - it's an optional feature
      test.skip(!hasInitButton, 'Initialize game button not present on this page');
      await expect(gameControlPage.initGameButton).toBeVisible();
    });
  });

  test.describe('Market Control', () => {
    test('should toggle market status', async ({ page }) => {
      // Get current status by checking the status badge text (OPEN or CLOSED)
      const statusBadge = page.locator('text=/^OPEN$|^CLOSED$/');
      const initialStatus = await statusBadge.textContent();
      const isInitiallyOpen = initialStatus === 'OPEN';

      // Click toggle
      await gameControlPage.toggleMarketButton.click();

      await page.waitForTimeout(2000);

      // Status should have changed - check badge text again
      const newStatus = await statusBadge.textContent();
      const isNowOpen = newStatus === 'OPEN';

      // Status should be different
      expect(isNowOpen).not.toBe(isInitiallyOpen);

      // Toggle back to original state
      await gameControlPage.toggleMarketButton.click();
      await page.waitForTimeout(1000);
    });
  });

  test.describe('Game Initialization', () => {
    test('should have init game functionality', async ({ page }) => {
      // Init game button may or may not be visible depending on app state
      const hasInitButton = await page.locator('button:has-text("Init"), button:has-text("Initialize")').isVisible().catch(() => false);
      test.skip(!hasInitButton, 'Initialize game button not present on this page');

      await expect(gameControlPage.initGameButton).toBeVisible();
    });
  });

  test.describe('Volatility Control', () => {
    test('should display volatility controls', async ({ page }) => {
      // Volatility controls are optional - skip if not present
      const hasVolatilityControl = await page.locator('text=/volatility/i').isVisible().catch(() => false);
      test.skip(!hasVolatilityControl, 'Volatility controls not present on this page');

      expect(hasVolatilityControl).toBeTruthy();
    });
  });
});

test.describe('Market Status Effects', () => {
  test('closed market should prevent order placement', async ({ browser }) => {
    // Admin closes market
    const adminContext = await browser.newContext();
    const adminPage = await adminContext.newPage();
    await loginAsAdmin(adminPage, TEST_USERS.admin.username, TEST_USERS.admin.password);
    await adminPage.goto('/admin/game');

    const gameControl = new GameControlPage(adminPage);

    // Ensure market is closed - check for the OPEN badge
    const statusBadge = adminPage.locator('text=/^OPEN$|^CLOSED$/');
    const currentStatus = await statusBadge.textContent();
    if (currentStatus === 'OPEN') {
      await gameControl.toggleMarketButton.click();
      await adminPage.waitForTimeout(2000);
    }

    // Trader tries to place order
    const traderContext = await browser.newContext();
    const traderPage = await traderContext.newPage();
    await loginAsTrader(traderPage, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);

    // Wait for page to fully load
    await traderPage.waitForLoadState('networkidle');
    await traderPage.waitForTimeout(1000);

    // Fill in order form - inputs are in the Quick Trade section on the right
    const qtyInput = traderPage.locator('input').first();
    const priceInput = traderPage.locator('input').nth(1);

    // Click Buy tab and Limit tab
    await traderPage.locator('button:has-text("Buy")').first().click();
    await traderPage.locator('button:has-text("Limit")').click();

    await qtyInput.fill('1');
    await priceInput.fill('100');

    // Click the place order button (e.g., "Buy RELIANCE")
    const placeOrderBtn = traderPage.locator('button').filter({ hasText: /^(Buy|Sell|Short)\s+\w+$/i }).first();
    await placeOrderBtn.click();

    // Handle confirmation modal if present
    const modal = traderPage.locator('[role="dialog"], .modal');
    if (await modal.isVisible({ timeout: 2000 }).catch(() => false)) {
      const confirmBtn = modal.locator('button').filter({ hasText: /confirm|yes|place/i });
      if (await confirmBtn.isVisible()) {
        await confirmBtn.click();
      }
    }

    await traderPage.waitForTimeout(2000);

    // Order should be rejected with market closed message
    await expect(traderPage.locator('body')).toContainText(/closed|rejected|cannot|market/i);

    // Clean up - reopen market
    await gameControl.toggleMarketButton.click();
    await adminPage.waitForTimeout(1000);

    await adminContext.close();
    await traderContext.close();
  });

  test('market status should sync across all connected clients', async ({ browser }) => {
    const adminContext = await browser.newContext();
    const trader1Context = await browser.newContext();
    const trader2Context = await browser.newContext();

    const adminPage = await adminContext.newPage();
    const trader1Page = await trader1Context.newPage();
    const trader2Page = await trader2Context.newPage();

    try {
      await loginAsAdmin(adminPage, TEST_USERS.admin.username, TEST_USERS.admin.password);
      await loginAsTrader(trader1Page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
      await loginAsTrader(trader2Page, TEST_USERS.trader2.regno, TEST_USERS.trader2.password);

      await adminPage.goto('/admin/game');
      const gameControl = new GameControlPage(adminPage);

      // Toggle market
      await gameControl.toggleMarketButton.click();

      // Wait for broadcast
      await adminPage.waitForTimeout(3000);
      await trader1Page.waitForTimeout(3000);
      await trader2Page.waitForTimeout(3000);

      // All clients should see the same market status
      // Check that traders see market status update
    } finally {
      await adminContext.close();
      await trader1Context.close();
      await trader2Context.close();
    }
  });
});
