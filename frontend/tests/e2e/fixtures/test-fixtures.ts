import { test as base, expect, Page, BrowserContext } from '@playwright/test';

// Test user credentials - Multiple users for parallel test execution
export const TEST_USERS = {
  // Primary test traders (for most tests)
  trader1: {
    regno: 'trader001',
    name: 'Test Trader One',
    password: 'password123',
  },
  trader2: {
    regno: 'REG123',
    name: 'Test Student',
    password: 'pass',
  },
  // Additional traders for parallel isolation
  trader3: {
    regno: 'trader003',
    name: 'Test Trader Three',
    password: 'password123',
  },
  trader4: {
    regno: 'trader004',
    name: 'Test Trader Four',
    password: 'password123',
  },
  trader5: {
    regno: 'trader005',
    name: 'Test Trader Five',
    password: 'password123',
  },
  // Traders for error handling tests
  errorTestTrader: {
    regno: 'errortest001',
    name: 'Error Test Trader',
    password: 'password123',
  },
  // Traders for portfolio tests
  portfolioTestTrader: {
    regno: 'portfolio001',
    name: 'Portfolio Test Trader',
    password: 'password123',
  },
  // Traders for order placement tests
  orderTestTrader: {
    regno: 'ordertest001',
    name: 'Order Test Trader',
    password: 'password123',
  },
  // Traders for multi-user tests
  multiUser1: {
    regno: 'multiuser001',
    name: 'Multi User One',
    password: 'password123',
  },
  multiUser2: {
    regno: 'multiuser002',
    name: 'Multi User Two',
    password: 'password123',
  },
  multiUser3: {
    regno: 'multiuser003',
    name: 'Multi User Three',
    password: 'password123',
  },
  admin: {
    username: 'admin',
    password: 'change_me_in_production',
  },
  newTrader: {
    regno: () => `NEW${Date.now().toString(36).toUpperCase()}`,
    name: 'New Test Trader',
    password: 'testpass123',
  },
};

// Generate unique trader for test isolation
export function generateUniqueTrader(prefix: string = 'TEST') {
  const timestamp = Date.now().toString(36).toUpperCase();
  const random = Math.random().toString(36).substring(2, 6).toUpperCase();
  return {
    regno: `${prefix}${timestamp}${random}`,
    name: `${prefix} User ${timestamp}`,
    password: 'testpass123',
  };
}

// Stock symbols
export const SYMBOLS = [
  'ZOMATO', 'TCS', 'TATAMOTORS', 'RELIANCE', 'HDFCBANK',
  'PAYTM', 'ADANIENT', 'SUNPHARMA', 'INFY', 'ICICIBANK'
];

// Price constants (PRICE_SCALE = 10000)
export const PRICE_SCALE = 10000;

// Helper functions for WebSocket message validation
export interface WebSocketMessage {
  type: string;
  payload: Record<string, unknown>;
}

// Extended test fixtures
interface TestFixtures {
  authenticatedTraderPage: Page;
  authenticatedAdminPage: Page;
  wsMessages: WebSocketMessage[];
  captureWsMessages: (page: Page) => Promise<() => WebSocketMessage[]>;
}

interface WorkerFixtures {
  traderContext: BrowserContext;
  adminContext: BrowserContext;
}

// Helper to wait for WebSocket connection
export async function waitForWsConnection(page: Page, timeout = 10000): Promise<void> {
  await page.waitForFunction(
    () => {
      const ws = (window as unknown as { websocketService?: { getConnectionStatus: () => boolean } }).websocketService;
      return ws?.getConnectionStatus?.() === true;
    },
    { timeout }
  );
}

// Helper to wait for specific server message
export async function waitForServerMessage(
  page: Page,
  messageType: string,
  timeout = 10000
): Promise<unknown> {
  return page.evaluate(
    ({ type, timeout: t }) => {
      return new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error(`Timeout waiting for ${type}`)), t);
        const ws = (window as unknown as { websocketService?: { on: (type: string, handler: (payload: unknown) => void) => () => void } }).websocketService;
        if (!ws) {
          clearTimeout(timer);
          reject(new Error('WebSocket service not found'));
          return;
        }
        const unsub = ws.on(type, (payload: unknown) => {
          clearTimeout(timer);
          unsub();
          resolve(payload);
        });
      });
    },
    { type: messageType, timeout }
  );
}

// Helper to send WebSocket message
export async function sendWsMessage(
  page: Page,
  message: { type: string; payload: Record<string, unknown> }
): Promise<boolean> {
  return page.evaluate((msg) => {
    const ws = (window as unknown as { websocketService?: { send: (msg: unknown) => boolean } }).websocketService;
    return ws?.send(msg) ?? false;
  }, message);
}

// Helper to get store state
export async function getAuthStoreState(page: Page) {
  return page.evaluate(() => {
    const state = (window as unknown as { useAuthStore?: { getState: () => unknown } }).useAuthStore?.getState();
    return state;
  });
}

export async function getGameStoreState(page: Page) {
  return page.evaluate(() => {
    const state = (window as unknown as { useGameStore?: { getState: () => unknown } }).useGameStore?.getState();
    return state;
  });
}

// Login helper with auto-registration fallback
export async function loginAsTrader(
  page: Page,
  regno: string,
  password: string,
  name?: string
): Promise<void> {
  await page.goto('/login');
  await page.waitForLoadState('networkidle');

  // Wait for login form to be visible
  await page.waitForSelector('input[placeholder*="registration" i]', { timeout: 10000 });

  // Fill login form - using actual placeholder text pattern
  await page.locator('input[placeholder*="registration" i]').fill(regno);
  await page.locator('input[placeholder*="password" i]').fill(password);

  // Click sign in
  await page.getByRole('button', { name: /sign in/i }).click();

  // Wait for navigation - either to trade page or error
  try {
    await page.waitForURL('/trade', { timeout: 10000 });
  } catch {
    // Check if there's an error message (user doesn't exist)
    const errorVisible = await page.locator('[role="alert"], .text-red-500, .text-red-400').isVisible().catch(() => false);

    if (errorVisible && name) {
      // User doesn't exist, try to register
      await registerNewTrader(page, regno, name, password);
    } else {
      // Wait again with longer timeout
      await page.waitForURL('/trade', { timeout: 15000 });
    }
  }
}

// Login or create trader - ensures user exists before login
export async function loginOrCreateTrader(
  page: Page,
  user: { regno: string; name: string; password: string }
): Promise<void> {
  await loginAsTrader(page, user.regno, user.password, user.name);
}

// Admin login helper
export async function loginAsAdmin(
  page: Page,
  username: string,
  password: string
): Promise<void> {
  await page.goto('/login');
  await page.waitForLoadState('networkidle');

  // Wait for login form to be visible
  await page.waitForSelector('button, [role="tab"]', { timeout: 10000 });

  // Click Admin Mode tab
  await page.locator('button:has-text("Admin"), [role="tab"]:has-text("Admin")').click();

  // Wait for admin form fields
  await page.waitForTimeout(500);

  // Fill admin login form - username field
  await page.locator('input').first().fill(username);
  await page.locator('input[type="password"]').fill(password);

  // Click admin login button (different text than trader sign in)
  await page.getByRole('button', { name: /admin login|sign in/i }).click();

  // Wait for navigation to admin dashboard
  await page.waitForURL(/\/admin/, { timeout: 15000 });
}

// Logout helper
export async function logout(page: Page): Promise<void> {
  // Click logout button in header
  const logoutBtn = page.getByRole('button', { name: /logout|sign out/i });
  if (await logoutBtn.isVisible()) {
    await logoutBtn.click();
    await page.waitForURL('/login', { timeout: 10000 });
  }
}

// Register new trader helper
export async function registerNewTrader(
  page: Page,
  regno: string,
  name: string,
  password: string
): Promise<void> {
  await page.goto('/register');
  await page.waitForLoadState('networkidle');

  // Wait for registration form to be visible
  await page.waitForSelector('input[placeholder*="registration" i]', { timeout: 10000 });

  // Fill registration form
  await page.locator('input[placeholder*="registration" i]').fill(regno);
  // Display name field has placeholder "How you'll appear to others"
  await page.locator('input[placeholder*="appear" i], input[placeholder*="display" i]').fill(name);
  await page.locator('input[type="password"]').first().fill(password);
  await page.locator('input[type="password"]').last().fill(password);

  // Click create account
  await page.getByRole('button', { name: /create account/i }).click();

  // Wait for navigation to trading page
  await page.waitForURL('/trade', { timeout: 15000 });
}

// Extended base test with fixtures
export const test = base.extend<TestFixtures, WorkerFixtures>({
  // Worker-scoped browser contexts for parallel multi-user tests
  traderContext: [async ({ browser }, use) => {
    const context = await browser.newContext({
      viewport: { width: 1920, height: 1080 },
    });
    await use(context);
    await context.close();
  }, { scope: 'worker' }],

  adminContext: [async ({ browser }, use) => {
    const context = await browser.newContext({
      viewport: { width: 1920, height: 1080 },
    });
    await use(context);
    await context.close();
  }, { scope: 'worker' }],

  // Pre-authenticated trader page
  authenticatedTraderPage: async ({ page }, use) => {
    await loginAsTrader(page, TEST_USERS.trader1.regno, TEST_USERS.trader1.password);
    await use(page);
  },

  // Pre-authenticated admin page
  authenticatedAdminPage: async ({ page }, use) => {
    await loginAsAdmin(page, TEST_USERS.admin.username, TEST_USERS.admin.password);
    await use(page);
  },

  // WebSocket message capture fixture
  wsMessages: async ({}, use) => {
    const messages: WebSocketMessage[] = [];
    await use(messages);
  },

  // Function to start capturing WS messages
  captureWsMessages: async ({}, use) => {
    const capturer = async (page: Page) => {
      const messages: WebSocketMessage[] = [];

      await page.evaluate(() => {
        const captured: Array<{ type: string; payload: unknown }> = [];
        (window as unknown as { __capturedWsMessages: typeof captured }).__capturedWsMessages = captured;

        const originalDispatch = (window as unknown as { websocketService?: { dispatch?: (type: string, payload: unknown) => void } }).websocketService?.dispatch;
        if (originalDispatch) {
          (window as unknown as { websocketService: { dispatch: (type: string, payload: unknown) => void } }).websocketService.dispatch = function(type: string, payload: unknown) {
            captured.push({ type, payload });
            return originalDispatch.call(this, type, payload);
          };
        }
      });

      return () => {
        return page.evaluate(() => {
          return (window as unknown as { __capturedWsMessages: WebSocketMessage[] }).__capturedWsMessages || [];
        });
      };
    };

    await use(capturer);
  },
});

export { expect };
