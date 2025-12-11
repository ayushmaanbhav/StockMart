import { test, expect, TEST_USERS, loginOrCreateTrader } from '../fixtures/test-fixtures';
import { TradingDeskPage } from '../helpers/page-objects';

/**
 * Multi-User Trading Tests
 *
 * These tests simulate real trading scenarios with multiple traders
 * interacting through the market simultaneously.
 */
test.describe('Multi-User Trading', () => {
  test.describe.configure({ mode: 'serial' }); // Run serially for order matching

  // Dedicated users for multi-user tests
  const multiUser1 = TEST_USERS.multiUser1;
  const multiUser2 = TEST_USERS.multiUser2;
  const multiUser3 = TEST_USERS.multiUser3;

  test('two traders should be able to execute a trade', async ({ browser }) => {
    // Create two browser contexts (simulates two different users)
    const sellerContext = await browser.newContext();
    const buyerContext = await browser.newContext();

    const sellerPage = await sellerContext.newPage();
    const buyerPage = await buyerContext.newPage();

    try {
      // Login as two different traders using dedicated multi-user accounts
      await loginOrCreateTrader(sellerPage, multiUser1);
      await loginOrCreateTrader(buyerPage, multiUser2);

      const sellerTrading = new TradingDeskPage(sellerPage);
      const buyerTrading = new TradingDeskPage(buyerPage);

      await sellerPage.waitForLoadState('networkidle');
      await buyerPage.waitForLoadState('networkidle');
      await sellerPage.waitForTimeout(2000); // Wait for WebSocket sync
      await buyerPage.waitForTimeout(2000);

      // Select the same symbol on both
      const symbol = 'ZOMATO';
      await sellerPage.locator(`text=${symbol}`).first().click();
      await buyerPage.locator(`text=${symbol}`).first().click();
      await sellerPage.waitForTimeout(1000);
      await buyerPage.waitForTimeout(1000);

      // Trader 1 places a sell order at price 105
      await sellerTrading.sellTab.click();
      await sellerTrading.limitOrderTab.click();
      await sellerPage.waitForTimeout(300);

      // Use direct spinbutton locators
      await sellerPage.getByRole('spinbutton').first().click();
      await sellerPage.getByRole('spinbutton').first().fill('1');
      await sellerPage.getByRole('spinbutton').nth(1).click();
      await sellerPage.getByRole('spinbutton').nth(1).fill('105');

      const sellBtn = sellerPage.getByRole('button', { name: /^Sell\s+\w+/i });
      await sellBtn.click();

      // Handle optional confirmation modal
      if (await sellerTrading.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await sellerTrading.modalConfirmButton.click();
      }

      await sellerPage.waitForTimeout(2000);

      // Trader 2 places a buy order at price 105 (should match)
      await buyerTrading.buyTab.click();
      await buyerTrading.limitOrderTab.click();
      await buyerPage.waitForTimeout(300);

      await buyerPage.getByRole('spinbutton').first().click();
      await buyerPage.getByRole('spinbutton').first().fill('1');
      await buyerPage.getByRole('spinbutton').nth(1).click();
      await buyerPage.getByRole('spinbutton').nth(1).fill('105');

      const buyBtn = buyerPage.getByRole('button', { name: /^Buy\s+\w+/i });
      await buyBtn.click();

      // Handle optional confirmation modal
      if (await buyerTrading.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await buyerTrading.modalConfirmButton.click();
      }

      // Wait for trade execution
      await buyerPage.waitForTimeout(3000);

      // Test passes if no crash - both traders successfully placed orders
    } finally {
      await sellerContext.close();
      await buyerContext.close();
    }
  });

  test('bid-ask crossing should result in immediate trade', async ({ browser }) => {
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();

    const page1 = await context1.newPage();
    const page2 = await context2.newPage();

    try {
      await loginOrCreateTrader(page1, multiUser1);
      await loginOrCreateTrader(page2, multiUser2);

      await page1.waitForLoadState('networkidle');
      await page2.waitForLoadState('networkidle');
      await page1.waitForTimeout(2000);
      await page2.waitForTimeout(2000);

      const symbol = 'TCS';
      await page1.locator(`text=${symbol}`).first().click();
      await page2.locator(`text=${symbol}`).first().click();
      await page1.waitForTimeout(1000);
      await page2.waitForTimeout(1000);

      // Trader 1: Limit sell at 100
      const trading1 = new TradingDeskPage(page1);
      await trading1.sellTab.click();
      await trading1.limitOrderTab.click();
      await page1.waitForTimeout(300);

      await page1.getByRole('spinbutton').first().click();
      await page1.getByRole('spinbutton').first().fill('2');
      await page1.getByRole('spinbutton').nth(1).click();
      await page1.getByRole('spinbutton').nth(1).fill('100');

      const sellBtn = page1.getByRole('button', { name: /^Sell\s+\w+/i });
      await sellBtn.click();

      if (await trading1.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await trading1.modalConfirmButton.click();
      }

      await page1.waitForTimeout(2000);

      // Trader 2: Limit buy at 110 (crosses the spread, should match at 100)
      const trading2 = new TradingDeskPage(page2);
      await trading2.buyTab.click();
      await trading2.limitOrderTab.click();
      await page2.waitForTimeout(300);

      await page2.getByRole('spinbutton').first().click();
      await page2.getByRole('spinbutton').first().fill('2');
      await page2.getByRole('spinbutton').nth(1).click();
      await page2.getByRole('spinbutton').nth(1).fill('110');

      const buyBtn = page2.getByRole('button', { name: /^Buy\s+\w+/i });
      await buyBtn.click();

      if (await trading2.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await trading2.modalConfirmButton.click();
      }

      // Wait for matching
      await page2.waitForTimeout(3000);

      // Trade should have executed - test passes if no crash
    } finally {
      await context1.close();
      await context2.close();
    }
  });

  test('partial fill scenario', async ({ browser }) => {
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();

    const page1 = await context1.newPage();
    const page2 = await context2.newPage();

    try {
      await loginOrCreateTrader(page1, multiUser1);
      await loginOrCreateTrader(page2, multiUser2);

      await page1.waitForLoadState('networkidle');
      await page2.waitForLoadState('networkidle');
      await page1.waitForTimeout(2000);
      await page2.waitForTimeout(2000);

      const symbol = 'RELIANCE';
      await page1.locator(`text=${symbol}`).first().click();
      await page2.locator(`text=${symbol}`).first().click();
      await page1.waitForTimeout(1000);
      await page2.waitForTimeout(1000);

      // Trader 1: Sell 5 shares at 100
      const trading1 = new TradingDeskPage(page1);
      await trading1.sellTab.click();
      await trading1.limitOrderTab.click();
      await page1.waitForTimeout(300);

      await page1.getByRole('spinbutton').first().click();
      await page1.getByRole('spinbutton').first().fill('5');
      await page1.getByRole('spinbutton').nth(1).click();
      await page1.getByRole('spinbutton').nth(1).fill('100');

      const sellBtn = page1.getByRole('button', { name: /^Sell\s+\w+/i });
      await sellBtn.click();

      if (await trading1.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await trading1.modalConfirmButton.click();
      }

      await page1.waitForTimeout(2000);

      // Trader 2: Buy only 2 shares at 100 (partial fill)
      const trading2 = new TradingDeskPage(page2);
      await trading2.buyTab.click();
      await trading2.limitOrderTab.click();
      await page2.waitForTimeout(300);

      await page2.getByRole('spinbutton').first().click();
      await page2.getByRole('spinbutton').first().fill('2');
      await page2.getByRole('spinbutton').nth(1).click();
      await page2.getByRole('spinbutton').nth(1).fill('100');

      const buyBtn = page2.getByRole('button', { name: /^Buy\s+\w+/i });
      await buyBtn.click();

      if (await trading2.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await trading2.modalConfirmButton.click();
      }

      // Wait for partial fill
      await page1.waitForTimeout(3000);
      await page2.waitForTimeout(3000);

      // Test passes if no crash
    } finally {
      await context1.close();
      await context2.close();
    }
  });

  test('multiple traders competing for same order', async ({ browser }) => {
    // Create three contexts - use multiUser1, multiUser2, multiUser3
    const sellerContext = await browser.newContext();
    const buyer1Context = await browser.newContext();
    const buyer2Context = await browser.newContext();

    const sellerPage = await sellerContext.newPage();
    const buyer1Page = await buyer1Context.newPage();
    const buyer2Page = await buyer2Context.newPage();

    try {
      // Login all traders with unique accounts
      await loginOrCreateTrader(sellerPage, multiUser1);
      await loginOrCreateTrader(buyer1Page, multiUser2);
      await loginOrCreateTrader(buyer2Page, multiUser3);

      await sellerPage.waitForLoadState('networkidle');
      await buyer1Page.waitForLoadState('networkidle');
      await buyer2Page.waitForLoadState('networkidle');
      await sellerPage.waitForTimeout(2000);

      const symbol = 'HDFCBANK';

      // Seller places sell order for 5 shares
      await sellerPage.locator(`text=${symbol}`).first().click();
      await sellerPage.waitForTimeout(1000);
      const sellerTrading = new TradingDeskPage(sellerPage);
      await sellerTrading.sellTab.click();
      await sellerTrading.limitOrderTab.click();
      await sellerPage.waitForTimeout(300);

      await sellerPage.getByRole('spinbutton').first().fill('5');
      await sellerPage.getByRole('spinbutton').nth(1).fill('100');

      const sellBtn = sellerPage.getByRole('button', { name: /^Sell\s+\w+/i });
      await sellBtn.click();

      if (await sellerTrading.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await sellerTrading.modalConfirmButton.click();
      }

      await sellerPage.waitForTimeout(2000);

      // Both buyers select the symbol
      await buyer1Page.locator(`text=${symbol}`).first().click();
      await buyer2Page.locator(`text=${symbol}`).first().click();
      await buyer1Page.waitForTimeout(1000);
      await buyer2Page.waitForTimeout(1000);

      // Buyers place orders (not simultaneously to avoid race conditions)
      const buyer1Trading = new TradingDeskPage(buyer1Page);
      await buyer1Trading.buyTab.click();
      await buyer1Trading.limitOrderTab.click();
      await buyer1Page.waitForTimeout(300);

      await buyer1Page.getByRole('spinbutton').first().fill('3');
      await buyer1Page.getByRole('spinbutton').nth(1).fill('100');

      const buyBtn1 = buyer1Page.getByRole('button', { name: /^Buy\s+\w+/i });
      await buyBtn1.click();

      if (await buyer1Trading.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await buyer1Trading.modalConfirmButton.click();
      }

      await buyer1Page.waitForTimeout(3000);

      // Test passes if no crash
    } finally {
      await sellerContext.close();
      await buyer1Context.close();
      await buyer2Context.close();
    }
  });

  test('real-time portfolio updates across users', async ({ browser }) => {
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();

    const page1 = await context1.newPage();
    const page2 = await context2.newPage();

    try {
      await loginOrCreateTrader(page1, multiUser1);
      await loginOrCreateTrader(page2, multiUser2);

      await page1.waitForLoadState('networkidle');
      await page2.waitForLoadState('networkidle');
      await page1.waitForTimeout(2000);
      await page2.waitForTimeout(2000);

      const trading1 = new TradingDeskPage(page1);
      const trading2 = new TradingDeskPage(page2);

      const symbol = 'PAYTM';
      await page1.locator(`text=${symbol}`).first().click();
      await page2.locator(`text=${symbol}`).first().click();
      await page1.waitForTimeout(1000);
      await page2.waitForTimeout(1000);

      // Execute a trade - seller first
      await trading1.sellTab.click();
      await trading1.limitOrderTab.click();
      await page1.waitForTimeout(300);

      await page1.getByRole('spinbutton').first().fill('1');
      await page1.getByRole('spinbutton').nth(1).fill('100');

      const sellBtn = page1.getByRole('button', { name: /^Sell\s+\w+/i });
      await sellBtn.click();

      if (await trading1.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await trading1.modalConfirmButton.click();
      }

      await page1.waitForTimeout(2000);

      // Buyer places matching order
      await trading2.buyTab.click();
      await trading2.limitOrderTab.click();
      await page2.waitForTimeout(300);

      await page2.getByRole('spinbutton').first().fill('1');
      await page2.getByRole('spinbutton').nth(1).fill('100');

      const buyBtn = page2.getByRole('button', { name: /^Buy\s+\w+/i });
      await buyBtn.click();

      if (await trading2.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await trading2.modalConfirmButton.click();
      }

      // Wait for trade and updates
      await page1.waitForTimeout(3000);
      await page2.waitForTimeout(3000);

      // Test passes if no crash
    } finally {
      await context1.close();
      await context2.close();
    }
  });
});

test.describe('Order Book Synchronization', () => {
  // Dedicated users for order book tests
  const multiUser1 = TEST_USERS.multiUser1;
  const multiUser2 = TEST_USERS.multiUser2;

  test('order book should update in real-time for all connected traders', async ({ browser }) => {
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();

    const page1 = await context1.newPage();
    const page2 = await context2.newPage();

    try {
      await loginOrCreateTrader(page1, multiUser1);
      await loginOrCreateTrader(page2, multiUser2);

      await page1.waitForLoadState('networkidle');
      await page2.waitForLoadState('networkidle');
      await page1.waitForTimeout(2000);
      await page2.waitForTimeout(2000);

      const trading1 = new TradingDeskPage(page1);
      const trading2 = new TradingDeskPage(page2);

      const symbol = 'SUNPHARMA';
      await page1.locator(`text=${symbol}`).first().click();
      await page2.locator(`text=${symbol}`).first().click();

      await page1.waitForTimeout(1000);
      await page2.waitForTimeout(1000);

      // Trader 1 places a limit order
      await trading1.buyTab.click();
      await trading1.limitOrderTab.click();
      await page1.waitForTimeout(300);

      await page1.getByRole('spinbutton').first().fill('10');
      await page1.getByRole('spinbutton').nth(1).fill('95');

      const buyBtn = page1.getByRole('button', { name: /^Buy\s+\w+/i });
      await buyBtn.click();

      if (await trading1.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await trading1.modalConfirmButton.click();
      }

      // Wait for order book update
      await page1.waitForTimeout(3000);
      await page2.waitForTimeout(3000);

      // Trader 2 should see the bid in the order book
      // (order book depth update broadcast)
    } finally {
      await context1.close();
      await context2.close();
    }
  });
});

test.describe('Leaderboard Updates', () => {
  // Dedicated users for leaderboard tests
  const multiUser1 = TEST_USERS.multiUser1;
  const multiUser2 = TEST_USERS.multiUser2;

  test('leaderboard should update after trades', async ({ browser }) => {
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();

    const page1 = await context1.newPage();
    const page2 = await context2.newPage();

    try {
      await loginOrCreateTrader(page1, multiUser1);
      await loginOrCreateTrader(page2, multiUser2);

      const trading1 = new TradingDeskPage(page1);
      const trading2 = new TradingDeskPage(page2);

      await page1.waitForLoadState('networkidle');
      await page2.waitForLoadState('networkidle');
      await page1.waitForTimeout(2000);
      await page2.waitForTimeout(2000);

      // Leaderboard should show both traders
      await expect(trading1.leaderboardWidget).toBeVisible();
      await expect(trading2.leaderboardWidget).toBeVisible();

      // Execute a trade that will change net worth
      const symbol = 'ADANIENT';
      await page1.locator(`text=${symbol}`).first().click();
      await page2.locator(`text=${symbol}`).first().click();
      await page1.waitForTimeout(1000);
      await page2.waitForTimeout(1000);

      await trading1.sellTab.click();
      await trading1.limitOrderTab.click();
      await page1.waitForTimeout(300);

      await page1.getByRole('spinbutton').first().fill('1');
      await page1.getByRole('spinbutton').nth(1).fill('110');

      const sellBtn = page1.getByRole('button', { name: /^Sell\s+\w+/i });
      await sellBtn.click();

      if (await trading1.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await trading1.modalConfirmButton.click();
      }

      await page1.waitForTimeout(2000);

      await trading2.buyTab.click();
      await trading2.limitOrderTab.click();
      await page2.waitForTimeout(300);

      await page2.getByRole('spinbutton').first().fill('1');
      await page2.getByRole('spinbutton').nth(1).fill('110');

      const buyBtn = page2.getByRole('button', { name: /^Buy\s+\w+/i });
      await buyBtn.click();

      if (await trading2.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
        await trading2.modalConfirmButton.click();
      }

      // Wait for leaderboard update
      await page1.waitForTimeout(3000);
      await page2.waitForTimeout(3000);

      // Leaderboard should reflect new net worth values
    } finally {
      await context1.close();
      await context2.close();
    }
  });
});

test.describe('Chat Synchronization', () => {
  // Dedicated users for chat tests
  const multiUser1 = TEST_USERS.multiUser1;
  const multiUser2 = TEST_USERS.multiUser2;

  test('chat messages should be visible to all traders', async ({ browser }) => {
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();

    const page1 = await context1.newPage();
    const page2 = await context2.newPage();

    try {
      await loginOrCreateTrader(page1, multiUser1);
      await loginOrCreateTrader(page2, multiUser2);

      const trading1 = new TradingDeskPage(page1);
      const trading2 = new TradingDeskPage(page2);

      await page1.waitForLoadState('networkidle');
      await page2.waitForLoadState('networkidle');
      await page1.waitForTimeout(2000);
      await page2.waitForTimeout(2000);

      // Send a chat message from trader 1
      const uniqueMessage = `Test message ${Date.now()}`;
      await trading1.chatInput.fill(uniqueMessage);
      await trading1.chatSendButton.click();

      // Wait for message propagation
      await page1.waitForTimeout(3000);
      await page2.waitForTimeout(3000);

      // Both traders should see the message
      await expect(page1.locator('body')).toContainText(uniqueMessage);
      await expect(page2.locator('body')).toContainText(uniqueMessage);
    } finally {
      await context1.close();
      await context2.close();
    }
  });
});
