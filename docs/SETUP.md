# StockMart Setup Guide

Welcome to StockMart! This guide will help you get the trading simulation up and running on your computer. Whether you're a student learning to trade, an educator setting up a classroom lab, or a developer exploring the codebase, this guide has you covered.

## Table of Contents

- [Quick Start (5 Minutes)](#quick-start-5-minutes)
- [Detailed Setup](#detailed-setup)
  - [Installing Rust](#installing-rust)
  - [Installing Node.js](#installing-nodejs)
  - [Running the Application](#running-the-application)
- [Your First Trade](#your-first-trade)
- [Troubleshooting](#troubleshooting)
- [For Educators](#for-educators)
- [FAQ](#faq)

---

## Quick Start (5 Minutes)

If you're experienced with development tools, here's the fast path:

```bash
# Prerequisites: Rust 1.76+, Node.js 18+

# Clone the repository
git clone https://github.com/yourusername/stockmart.git
cd stockmart

# Terminal 1: Start backend
cd backend && RUST_LOG=info cargo run

# Terminal 2: Start frontend
cd frontend && npm install && npm run dev

# Open http://localhost:5174 in your browser
```

Need more details? Keep reading!

---

## Detailed Setup

### Step 1: Check Your System

StockMart works on:
- **macOS** (Intel or Apple Silicon)
- **Windows** 10/11
- **Linux** (Ubuntu, Fedora, etc.)

You'll need about **2GB of free disk space** and a modern web browser (Chrome, Firefox, Safari, or Edge).

---

### Installing Rust

Rust is the programming language used for StockMart's backend. Don't worry if you've never used Rust before - you just need to install it!

#### macOS / Linux

Open Terminal and run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When prompted, press **Enter** to proceed with the default installation.

After installation, restart your terminal or run:

```bash
source $HOME/.cargo/env
```

#### Windows

1. Download the Rust installer from [rustup.rs](https://rustup.rs/)
2. Run the downloaded `.exe` file
3. Follow the installer prompts (defaults are fine)
4. You may need to install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

#### Verify Installation

```bash
rustc --version
```

You should see something like:
```
rustc 1.76.0 (07dca489a 2024-02-04)
```

---

### Installing Node.js

Node.js is used for StockMart's frontend. We recommend using the LTS (Long Term Support) version.

#### Option 1: Direct Download (Easiest)

1. Go to [nodejs.org](https://nodejs.org/)
2. Download the **LTS** version
3. Run the installer
4. Follow the prompts (defaults are fine)

#### Option 2: Using nvm (Recommended for developers)

**macOS / Linux:**
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc  # or ~/.zshrc on macOS
nvm install --lts
```

**Windows:**
Download [nvm-windows](https://github.com/coreybutler/nvm-windows/releases), then:
```powershell
nvm install lts
nvm use lts
```

#### Verify Installation

```bash
node --version
npm --version
```

You should see something like:
```
v20.10.0
10.2.3
```

---

### Running the Application

#### Step 1: Clone the Repository

```bash
git clone https://github.com/yourusername/stockmart.git
cd stockmart
```

#### Step 2: Start the Backend

Open a terminal window and run:

```bash
cd backend
RUST_LOG=info cargo run
```

**First time?** This will download and compile dependencies. It may take 2-5 minutes. Subsequent runs are much faster!

You'll see output like:
```
   Compiling stockmart-backend v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 2m 30s
     Running `target/debug/stockmart-backend`
2024-01-15T10:30:00.000Z  INFO stockmart: Starting StockMart backend on port 3000
2024-01-15T10:30:00.100Z  INFO stockmart: WebSocket server ready
```

**Keep this terminal open!** The backend needs to stay running.

#### Step 3: Start the Frontend

Open a **new** terminal window and run:

```bash
cd frontend
npm install  # Only needed the first time
npm run dev
```

You'll see:
```
  VITE v5.0.0  ready in 500 ms

  ➜  Local:   http://localhost:5174/
  ➜  Network: use --host to expose
```

#### Step 4: Open the Application

Open your web browser and go to:

**[http://localhost:5174](http://localhost:5174)**

You should see the StockMart login page!

---

## Your First Trade

Let's walk through placing your first trade:

### 1. Create an Account

1. Click **"Register here"** on the login page
2. Enter a registration number (like your student ID)
3. Enter your display name
4. Create a password
5. Click **"Create Account"**

You'll start with **$100,000** in virtual cash!

### 2. Explore the Trading Desk

After logging in, you'll see the trading desk with:

- **Chart**: Shows price history for the selected stock
- **Order Book**: Shows current buy and sell orders
- **Portfolio**: Your holdings and available cash
- **Leaderboard**: Top traders by net worth
- **Quick Trade**: Place orders here

### 3. Place a Buy Order

1. In the **Quick Trade** widget, make sure **"Buy"** is selected
2. Select **"Limit"** for order type
3. Enter quantity: `10`
4. Enter price: `100` (or whatever the current price is)
5. Click **"Buy [SYMBOL]"**
6. Confirm the order in the popup

Congratulations! You've placed your first order!

### 4. Check Your Portfolio

Look at the **Portfolio** section to see:
- Your position in the stock
- Current value
- Profit/Loss (P&L)

### 5. Try Selling

To sell shares you own:
1. Click **"Sell"** in Quick Trade
2. Enter quantity (up to what you own)
3. Set a price
4. Submit the order

---

## Troubleshooting

### Backend Won't Start

**Error: `cargo: command not found`**
- Rust isn't installed or not in your PATH
- Try restarting your terminal
- Re-run the Rust installation

**Error: `Address already in use`**
- Port 3000 is being used by another program
- Kill the other process or wait for it to finish
- On macOS/Linux: `lsof -i :3000` to find what's using it

**Error: Compilation errors**
- Make sure you have the latest Rust: `rustup update`
- Try cleaning and rebuilding: `cargo clean && cargo build`

### Frontend Won't Start

**Error: `npm: command not found`**
- Node.js isn't installed or not in your PATH
- Try restarting your terminal
- Re-run the Node.js installation

**Error: `EACCES permission denied`**
- On macOS/Linux, you may need to fix npm permissions
- See: [npm docs](https://docs.npmjs.com/resolving-eacces-permissions-errors-when-installing-packages-globally)

**Error: `Cannot connect to WebSocket`**
- Make sure the backend is running first
- Check that the backend is on port 3000
- Try refreshing the page

### Page is Blank or Broken

1. Open browser developer tools (F12)
2. Check the Console tab for errors
3. Try clearing your browser cache
4. Try a different browser

### Can't Log In

- **"User not found"**: Registration number doesn't exist - register first
- **"Invalid password"**: Check your password is correct
- **Connection error**: Make sure backend is running

---

## For Educators

### Setting Up a Classroom

1. **One Backend, Many Clients**: Run one backend server that all students connect to
2. **Network Access**: Use `cargo run -- --host 0.0.0.0` to allow network connections
3. **Frontend Config**: Students can use the hosted frontend or run their own

### Starting a Trading Session

1. Log in as admin (default: admin/admin)
2. Go to **Game Control**
3. Click **"Initialize Game"** to reset all portfolios
4. Set the **Market** to **Open**
5. Students can now trade!

### Running a Competition

1. Initialize the game with equal starting capital
2. Set a time limit (e.g., 30 minutes)
3. Students trade to maximize their net worth
4. Close the market when time is up
5. Check the leaderboard for winners!

### Customizing for Your Class

**Change starting capital:**
Edit `backend/data/config.json`:
```json
{
  "default_starting_money": 100000,
  "default_shares_per_company": 50
}
```

**Add custom companies:**
Use the Admin Dashboard → Companies → New Company (IPO)

---

## FAQ

### General Questions

**Q: Is this real money?**
A: No! StockMart uses virtual money only. It's a simulation for learning.

**Q: Can I lose more money than I have?**
A: No. The system prevents you from spending more than your available cash.

**Q: What happens if I close my browser?**
A: Your account and positions are saved. Just log back in to continue.

### Trading Questions

**Q: What's the difference between Market and Limit orders?**
A:
- **Market**: Executes immediately at the best available price
- **Limit**: Only executes at your specified price or better

**Q: What is short selling?**
A: Borrowing shares to sell, hoping to buy them back cheaper later. Requires 150% margin.

**Q: Why was my order rejected?**
A: Common reasons:
- Insufficient funds (buy orders)
- Insufficient shares (sell orders)
- Market is closed
- Price is invalid

**Q: What's a circuit breaker?**
A: A safety mechanism that halts trading when a stock moves too much (10%) in a short time.

### Technical Questions

**Q: Where is my data stored?**
A: In JSON files in the `backend/data/` directory. Data auto-saves every 60 seconds.

**Q: Can I reset my account?**
A: Ask an admin to initialize a new game, or delete your entry from `users.json` and re-register.

**Q: How do I run tests?**
A:
```bash
# Backend tests
cd backend && cargo test

# Frontend E2E tests
cd frontend && npm test
```

---

## Getting Help

- **GitHub Issues**: Report bugs or request features
- **Documentation**: Check the [Architecture Guide](./ARCHITECTURE.md)
- **Contributing**: See [Contributing Guide](./CONTRIBUTING.md)

---

Happy trading! Remember: in StockMart, the only thing you can lose is fake money - so experiment freely and learn from your mistakes!
