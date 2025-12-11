import { Page, Locator, expect } from '@playwright/test';

/**
 * Page Object for Login Page
 */
export class LoginPage {
  readonly page: Page;
  readonly traderTab: Locator;
  readonly adminTab: Locator;
  readonly regnoInput: Locator;
  readonly usernameInput: Locator;
  readonly passwordInput: Locator;
  readonly signInButton: Locator;
  readonly registerLink: Locator;
  readonly errorMessage: Locator;

  constructor(page: Page) {
    this.page = page;
    this.traderTab = page.locator('button:has-text("Trader"), [role="tab"]:has-text("Trader")');
    this.adminTab = page.locator('button:has-text("Admin"), [role="tab"]:has-text("Admin")');
    this.regnoInput = page.locator('input[placeholder*="registration" i]');
    this.usernameInput = page.locator('input[placeholder*="username" i], input[placeholder*="admin" i]').first();
    this.passwordInput = page.locator('input[placeholder*="password" i]');
    this.signInButton = page.getByRole('button', { name: /sign in/i });
    this.registerLink = page.locator('a:has-text("Register"), a:has-text("register"), a:has-text("Create")');
    this.errorMessage = page.locator('[role="alert"], .error-message, .text-red-500, .text-red-400');
  }

  async goto() {
    await this.page.goto('/login');
    await this.page.waitForLoadState('networkidle');
  }

  async loginAsTrader(regno: string, password: string) {
    await this.traderTab.click();
    await this.regnoInput.fill(regno);
    await this.passwordInput.fill(password);
    await this.signInButton.click();
  }

  async loginAsAdmin(username: string, password: string) {
    await this.adminTab.click();
    await this.page.waitForTimeout(500); // Wait for tab switch
    await this.usernameInput.fill(username);
    await this.passwordInput.fill(password);
    // Admin mode has different button text
    await this.page.getByRole('button', { name: /admin login|sign in/i }).click();
  }

  async expectError(message: string | RegExp) {
    await expect(this.errorMessage).toContainText(message);
  }
}

/**
 * Page Object for Register Page
 */
export class RegisterPage {
  readonly page: Page;
  readonly regnoInput: Locator;
  readonly nameInput: Locator;
  readonly passwordInput: Locator;
  readonly confirmPasswordInput: Locator;
  readonly createAccountButton: Locator;
  readonly loginLink: Locator;
  readonly errorMessage: Locator;

  constructor(page: Page) {
    this.page = page;
    this.regnoInput = page.locator('input[placeholder*="registration" i]');
    // Display name field has placeholder "How you'll appear to others"
    this.nameInput = page.locator('input[placeholder*="appear" i], input[placeholder*="display" i]');
    this.passwordInput = page.locator('input[type="password"]').first();
    this.confirmPasswordInput = page.locator('input[type="password"]').last();
    this.createAccountButton = page.getByRole('button', { name: /create account/i });
    this.loginLink = page.locator('a:has-text("Sign in"), a:has-text("Login"), a:has-text("login")');
    this.errorMessage = page.locator('[role="alert"], .error-message, .text-red-500, .text-red-400');
  }

  async goto() {
    await this.page.goto('/register');
    await this.page.waitForLoadState('networkidle');
  }

  async register(regno: string, name: string, password: string, confirmPassword?: string) {
    await this.regnoInput.fill(regno);
    await this.nameInput.fill(name);
    await this.passwordInput.fill(password);
    await this.confirmPasswordInput.fill(confirmPassword ?? password);
    await this.createAccountButton.click();
  }

  async expectError(message: string | RegExp) {
    await expect(this.errorMessage).toContainText(message);
  }
}

/**
 * Page Object for Trading Desk Page
 */
export class TradingDeskPage {
  readonly page: Page;

  // Header
  readonly connectionStatus: Locator;
  readonly userNameDisplay: Locator;
  readonly logoutButton: Locator;

  // Symbol Selector (dropdown in Quick Trade)
  readonly symbolSelector: Locator;
  readonly symbolSearch: Locator;

  // Quick Trade Widget - based on actual UI
  readonly buyTab: Locator;
  readonly sellTab: Locator;
  readonly shortTab: Locator;
  readonly marketOrderTab: Locator;
  readonly limitOrderTab: Locator;
  readonly quantityInput: Locator;
  readonly priceInput: Locator;
  readonly gtcButton: Locator;
  readonly iocButton: Locator;
  readonly orderTotal: Locator;
  readonly placeOrderButton: Locator;

  // Order Confirmation Modal
  readonly confirmationModal: Locator;
  readonly modalConfirmButton: Locator;
  readonly modalCancelButton: Locator;

  // Portfolio Widget
  readonly portfolioWidget: Locator;
  readonly moneyDisplay: Locator;
  readonly netWorthDisplay: Locator;
  readonly positionRows: Locator;

  // Order Book Widget
  readonly orderBookWidget: Locator;
  readonly bidRows: Locator;
  readonly askRows: Locator;

  // Open Orders Widget
  readonly openOrdersWidget: Locator;
  readonly orderRows: Locator;

  // Leaderboard Widget
  readonly leaderboardWidget: Locator;
  readonly leaderboardEntries: Locator;

  // Chat Widget
  readonly chatWidget: Locator;
  readonly chatInput: Locator;
  readonly chatSendButton: Locator;
  readonly chatMessages: Locator;

  // News Ticker
  readonly newsTicker: Locator;

  // Market Indices
  readonly marketIndices: Locator;

  // Chart
  readonly chart: Locator;

  constructor(page: Page) {
    this.page = page;

    // Header
    this.connectionStatus = page.locator('text=Live').or(page.locator('.connection-status'));
    this.userNameDisplay = page.locator('header').locator('text=/Test|Trader/i');
    this.logoutButton = page.locator('header button[aria-label*="logout" i], header svg').last();

    // Symbol Selector (dropdown showing current symbol - look for the dropdown in Quick Trade)
    this.symbolSelector = page.locator('button:has-text("RELIANCE"), button:has-text("TCS"), button:has-text("ZOMATO")').first();
    this.symbolSearch = page.getByPlaceholder(/search|symbol/i);

    // Quick Trade Widget - matches actual UI (in right sidebar)
    this.buyTab = page.locator('button').filter({ hasText: /^Buy$/ }).first();
    this.sellTab = page.locator('button').filter({ hasText: /^Sell$/ }).first();
    this.shortTab = page.locator('button').filter({ hasText: /^Short$/ }).first();
    this.marketOrderTab = page.locator('button').filter({ hasText: /^Market$/ });
    this.limitOrderTab = page.locator('button').filter({ hasText: /^Limit$/ });

    // Input fields - Qty and Price inputs in Quick Trade section (right side)
    this.quantityInput = page.locator('input[type="number"], input').nth(0);
    this.priceInput = page.locator('input[type="number"], input').nth(1);

    // Time in Force buttons (toggle buttons, not radio)
    this.gtcButton = page.locator('button').filter({ hasText: /^GTC$/ });
    this.iocButton = page.locator('button').filter({ hasText: /^IOC$/ });

    // Order total and place order
    this.orderTotal = page.locator('text=/Est\\. Total|Total/i');
    // The button text is "Buy SYMBOL" or "Sell SYMBOL" or "Short SYMBOL"
    this.placeOrderButton = page.locator('button').filter({ hasText: /^(Buy|Sell|Short)\s+\w+$/i }).first().or(
      page.locator('button:has-text("Place"), button:has-text("Confirm"), button:has-text("Submit")')
    );

    // Order Confirmation Modal
    this.confirmationModal = page.locator('[role="dialog"], .modal, .fixed.inset-0');
    this.modalConfirmButton = page.locator('[role="dialog"] button, .modal button').filter({ hasText: /confirm|yes|place/i });
    this.modalCancelButton = page.locator('[role="dialog"] button, .modal button').filter({ hasText: /cancel|no/i });

    // Portfolio Widget - matches actual "Portfolio" section (look for the widget with Portfolio title)
    this.portfolioWidget = page.locator('.widget-title:has-text("Portfolio")').locator('..').locator('..');
    this.moneyDisplay = page.locator('text=/CASH|Cash/i').locator('~ *').first();
    this.netWorthDisplay = page.locator('text=/NET WORTH|Net Worth/i').locator('~ *').first();
    this.positionRows = page.getByText('Positions', { exact: true }).locator('..').locator('div:has-text("@")');

    // Order Book Widget
    this.orderBookWidget = page.getByText('Order Book').locator('..').locator('..');
    this.bidRows = page.getByText('Bids').locator('..').locator('div');
    this.askRows = page.getByText('Asks').locator('..').locator('div');

    // Open Orders - look for the Open Orders text in widget header
    this.openOrdersWidget = page.getByText('Open Orders').locator('..');
    this.orderRows = page.locator('.order-row, [data-testid="order-row"]');

    // Leaderboard Widget
    this.leaderboardWidget = page.getByText('Leaderboard').locator('..');
    this.leaderboardEntries = this.leaderboardWidget.locator('> div').filter({ hasText: /₹/ });

    // Chat Widget
    this.chatWidget = page.locator('text=Chat').locator('..').locator('..');
    this.chatInput = page.getByPlaceholder(/message/i);
    this.chatSendButton = page.locator('button:has-text("Send")');
    this.chatMessages = this.chatWidget.locator('.message, [class*="message"]');

    // News Ticker
    this.newsTicker = page.locator('text=/NEWS/i').locator('..');

    // Market Indices - the scrolling bar at top
    this.marketIndices = page.locator('text=/LIVE|CONGLOMERATE|HEALTHCARE/i').first().locator('..');

    // Chart - TradingView chart
    this.chart = page.locator('.tv-lightweight-charts, [class*="chart"]').first();
  }

  // Backwards compatibility aliases
  get confirmOrderButton() { return this.placeOrderButton; }
  get gtcRadio() { return this.gtcButton; }
  get iocRadio() { return this.iocButton; }

  async goto() {
    await this.page.goto('/trade');
    await this.page.waitForLoadState('networkidle');
  }

  async selectSymbol(symbol: string) {
    // Click on symbol in the list or selector
    await this.page.locator(`text=${symbol}`).first().click();
    await this.page.waitForTimeout(500);
  }

  async fillOrderForm(qty: number, price?: number) {
    // Clear and fill quantity
    const qtyInput = this.page.locator('input').nth(0);
    await qtyInput.clear();
    await qtyInput.fill(qty.toString());

    // Fill price if provided (for limit orders)
    if (price !== undefined) {
      const priceInput = this.page.locator('input').nth(1);
      await priceInput.clear();
      await priceInput.fill(price.toString());
    }
  }

  async placeLimitBuyOrder(qty: number, price: number) {
    await this.buyTab.click();
    await this.limitOrderTab.click();
    await this.fillOrderForm(qty, price);

    // Look for any submit/confirm/place button
    const submitBtn = this.page.locator('button').filter({ hasText: /place|confirm|submit|buy/i }).last();
    await submitBtn.click();

    // Check for and handle confirmation modal
    if (await this.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
      await this.modalConfirmButton.click();
    }
  }

  async placeLimitSellOrder(qty: number, price: number) {
    await this.sellTab.click();
    await this.limitOrderTab.click();
    await this.fillOrderForm(qty, price);

    const submitBtn = this.page.locator('button').filter({ hasText: /place|confirm|submit|sell/i }).last();
    await submitBtn.click();

    if (await this.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
      await this.modalConfirmButton.click();
    }
  }

  async placeMarketBuyOrder(qty: number) {
    await this.buyTab.click();
    await this.marketOrderTab.click();
    await this.fillOrderForm(qty);

    const submitBtn = this.page.locator('button').filter({ hasText: /place|confirm|submit|buy/i }).last();
    await submitBtn.click();

    if (await this.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
      await this.modalConfirmButton.click();
    }
  }

  async placeMarketSellOrder(qty: number) {
    await this.sellTab.click();
    await this.marketOrderTab.click();
    await this.fillOrderForm(qty);

    const submitBtn = this.page.locator('button').filter({ hasText: /place|confirm|submit|sell/i }).last();
    await submitBtn.click();

    if (await this.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
      await this.modalConfirmButton.click();
    }
  }

  async placeShortOrder(qty: number, price: number) {
    await this.shortTab.click();
    await this.limitOrderTab.click();
    await this.fillOrderForm(qty, price);

    const submitBtn = this.page.locator('button').filter({ hasText: /place|confirm|submit|short/i }).last();
    await submitBtn.click();

    if (await this.confirmationModal.isVisible({ timeout: 2000 }).catch(() => false)) {
      await this.modalConfirmButton.click();
    }
  }

  async cancelOrder(orderIndex: number = 0) {
    const cancelButton = this.page.locator('button:has-text("Cancel")').nth(orderIndex);
    if (await cancelButton.isVisible()) {
      await cancelButton.click();
    }
  }

  async sendChatMessage(message: string) {
    await this.chatInput.fill(message);
    await this.chatSendButton.click();
  }

  async clickOrderBookBid(index: number = 0) {
    await this.bidRows.nth(index).click();
  }

  async clickOrderBookAsk(index: number = 0) {
    await this.askRows.nth(index).click();
  }

  async getPortfolioValue(): Promise<string> {
    // Look for Net Worth value
    const netWorthElement = this.page.locator('text=/NET WORTH/i').locator('..').locator('text=/₹/');
    const text = await netWorthElement.first().textContent();
    return text ?? '0';
  }

  async getCashBalance(): Promise<string> {
    // Look for Cash value
    const cashElement = this.page.locator('text=/CASH/i').locator('..').locator('text=/₹/');
    const text = await cashElement.first().textContent();
    return text ?? '0';
  }

  async logout() {
    // Find logout button in header
    const logoutBtn = this.page.locator('button[title*="logout" i], button[aria-label*="logout" i], header button').last();
    await logoutBtn.click();
    await this.page.waitForURL('/login', { timeout: 10000 });
  }
}

/**
 * Page Object for Admin Dashboard Page
 */
export class AdminDashboardPage {
  readonly page: Page;

  // Navigation
  readonly dashboardLink: Locator;
  readonly gameControlLink: Locator;
  readonly tradersLink: Locator;
  readonly companiesLink: Locator;
  readonly tradesLink: Locator;
  readonly ordersLink: Locator;
  readonly orderbookLink: Locator;
  readonly diagnosticsLink: Locator;
  readonly logoutButton: Locator;

  // Dashboard Metrics
  readonly activeTraders: Locator;
  readonly totalTrades: Locator;
  readonly tradingVolume: Locator;
  readonly marketCap: Locator;
  readonly openOrders: Locator;

  constructor(page: Page) {
    this.page = page;

    // Navigation
    this.dashboardLink = page.getByRole('link', { name: /dashboard/i });
    this.gameControlLink = page.getByRole('link', { name: /game.*control/i });
    this.tradersLink = page.getByRole('link', { name: /traders/i });
    this.companiesLink = page.getByRole('link', { name: /companies/i });
    this.tradesLink = page.getByRole('link', { name: /trade.*history/i });
    this.ordersLink = page.getByRole('link', { name: /open.*orders/i });
    this.orderbookLink = page.getByRole('link', { name: /orderbook/i });
    this.diagnosticsLink = page.getByRole('link', { name: /diagnostics/i });
    this.logoutButton = page.getByRole('button', { name: /logout|sign out/i });

    // Dashboard Metrics
    this.activeTraders = page.locator('[data-testid="active-traders"]').or(page.locator(':text("Active Traders") + *'));
    this.totalTrades = page.locator('[data-testid="total-trades"]').or(page.locator(':text("Total Trades") + *'));
    this.tradingVolume = page.locator('[data-testid="trading-volume"]').or(page.locator(':text("Trading Volume") + *'));
    this.marketCap = page.locator('[data-testid="market-cap"]').or(page.locator(':text("Market Cap") + *'));
    this.openOrders = page.locator('[data-testid="open-orders"]').or(page.locator(':text("Open Orders") + *'));
  }

  async goto() {
    await this.page.goto('/admin');
    await this.page.waitForLoadState('networkidle');
  }

  async navigateToGameControl() {
    await this.gameControlLink.click();
    await this.page.waitForURL(/\/admin\/game/);
  }

  async navigateToTraders() {
    await this.tradersLink.click();
    await this.page.waitForURL(/\/admin\/traders/);
  }

  async navigateToCompanies() {
    await this.companiesLink.click();
    await this.page.waitForURL(/\/admin\/companies/);
  }

  async navigateToTrades() {
    await this.tradesLink.click();
    await this.page.waitForURL(/\/admin\/trades/);
  }

  async navigateToOrders() {
    await this.ordersLink.click();
    await this.page.waitForURL(/\/admin\/orders/);
  }

  async navigateToOrderbook() {
    await this.orderbookLink.click();
    await this.page.waitForURL(/\/admin\/orderbook/);
  }

  async navigateToDiagnostics() {
    await this.diagnosticsLink.click();
    await this.page.waitForURL(/\/admin\/diagnostics/);
  }

  async logout() {
    await this.logoutButton.click();
    await this.page.waitForURL('/login');
  }
}

/**
 * Page Object for Game Control Page
 */
export class GameControlPage {
  readonly page: Page;

  // Market Controls
  readonly marketStatusBadge: Locator;
  readonly toggleMarketButton: Locator;

  // Game Init
  readonly initGameButton: Locator;

  // Volatility
  readonly volatilitySlider: Locator;
  readonly volatilityInput: Locator;

  // Trading Hours
  readonly tradingStartInput: Locator;
  readonly tradingEndInput: Locator;
  readonly saveTradingHoursButton: Locator;

  // Circuit Breaker
  readonly circuitBreakerToggle: Locator;

  constructor(page: Page) {
    this.page = page;

    // Market Controls
    this.marketStatusBadge = page.locator('[data-testid="market-status"]').or(page.locator('.market-status'));
    this.toggleMarketButton = page.getByRole('button', { name: /open market|close market|toggle market/i });

    // Game Init
    this.initGameButton = page.getByRole('button', { name: /init.*game|initialize/i });

    // Volatility
    this.volatilitySlider = page.locator('input[type="range"]').first();
    this.volatilityInput = page.getByLabel(/volatility/i);

    // Trading Hours
    this.tradingStartInput = page.getByLabel(/start.*time|trading.*start/i);
    this.tradingEndInput = page.getByLabel(/end.*time|trading.*end/i);
    this.saveTradingHoursButton = page.getByRole('button', { name: /save.*hours|update.*hours/i });

    // Circuit Breaker
    this.circuitBreakerToggle = page.getByRole('switch', { name: /circuit.*breaker/i }).or(
      page.locator('[data-testid="circuit-breaker-toggle"]')
    );
  }

  async goto() {
    await this.page.goto('/admin/game');
    await this.page.waitForLoadState('networkidle');
  }

  async openMarket() {
    const isOpen = await this.page.locator('text=Market Open').isVisible();
    if (!isOpen) {
      await this.toggleMarketButton.click();
    }
  }

  async closeMarket() {
    const isOpen = await this.page.locator('text=Market Open').isVisible();
    if (isOpen) {
      await this.toggleMarketButton.click();
    }
  }

  async initializeGame() {
    await this.initGameButton.click();
  }

  async setVolatility(level: number) {
    await this.volatilitySlider.fill(level.toString());
  }
}

/**
 * Page Object for Traders Admin Page
 */
export class TradersAdminPage {
  readonly page: Page;

  readonly traderRows: Locator;
  readonly searchInput: Locator;

  constructor(page: Page) {
    this.page = page;
    this.traderRows = page.locator('[data-testid="trader-row"]').or(page.locator('tbody tr'));
    this.searchInput = page.getByPlaceholder(/search/i);
  }

  async goto() {
    await this.page.goto('/admin/traders');
    await this.page.waitForLoadState('networkidle');
  }

  async banTrader(traderName: string) {
    const row = this.page.locator(`tr:has-text("${traderName}")`);
    const banButton = row.getByRole('button', { name: /ban/i });
    await banButton.click();
  }

  async unbanTrader(traderName: string) {
    const row = this.page.locator(`tr:has-text("${traderName}")`);
    const unbanButton = row.getByRole('button', { name: /unban/i });
    await unbanButton.click();
  }

  async muteTrader(traderName: string) {
    const row = this.page.locator(`tr:has-text("${traderName}")`);
    const muteButton = row.getByRole('button', { name: /mute/i });
    await muteButton.click();
  }

  async unmuteTrader(traderName: string) {
    const row = this.page.locator(`tr:has-text("${traderName}")`);
    const unmuteButton = row.getByRole('button', { name: /unmute/i });
    await unmuteButton.click();
  }

  async searchTraders(query: string) {
    await this.searchInput.fill(query);
  }
}
