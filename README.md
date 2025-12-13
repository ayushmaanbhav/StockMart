<div align="center">

# StockMart

### Real-time Multiplayer Stock Trading Game Platform

### https://ayushmaanbhav.github.io/StockMart/

**The Ultimate Trading Competition Platform for Schools, Colleges & Gaming Events**

[![Game](https://img.shields.io/badge/🎮_Game-Trading%20Simulation-FF6B6B?style=for-the-badge)](#features)
[![Multiplayer](https://img.shields.io/badge/👥_Multiplayer-100%2B_Concurrent-4ECDC4?style=for-the-badge)](#scalability)
[![Configurable](https://img.shields.io/badge/⚙️_Highly-Configurable-9B59B6?style=for-the-badge)](#configuration)
[![Leaderboard](https://img.shields.io/badge/🏆_Live-Leaderboard-FFE66D?style=for-the-badge)](#features)

[![Backend Tests](https://img.shields.io/badge/backend%20tests-463%20passed-success?style=flat-square)](./backend/tests)
[![E2E Tests](https://img.shields.io/badge/e2e%20tests-136%20passed-success?style=flat-square)](./frontend/tests)
[![Test Coverage](https://img.shields.io/badge/coverage-85%25-brightgreen?style=flat-square)](#testing)
[![Test Lines](https://img.shields.io/badge/test%20lines-17%2C000%2B-blue?style=flat-square)](#testing)

[![Rust](https://img.shields.io/badge/Rust-1.76+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.9-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![Tokio](https://img.shields.io/badge/Tokio-async-purple?style=flat-square)](https://tokio.rs/)
[![WebSocket](https://img.shields.io/badge/WebSocket-real--time-green?style=flat-square)](#features)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=flat-square)](./LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](./docs/CONTRIBUTING.md)
[![Made with Love](https://img.shields.io/badge/Made%20with-♥-red?style=flat-square)](#)

<br />

<img src="docs/screenshots/trading-desk.png" alt="StockMart Trading Desk" width="800" />

*Professional trading desk with real-time charts, order book, portfolio tracking, and live leaderboard*

**Host trading competitions with 100+ concurrent players. Fully configurable. Zero risk.**

</div>

---

## Table of Contents

- [Why StockMart?](#why-stockmart)
- [For Event Organizers](#for-event-organizers)
- [For College Administrators](#for-college-administrators)
- [Game Modes](#game-modes)
- [Configuration](#configuration)
- [Scalability](#scalability)
- [Quick Start](#quick-start)
- [Features](#features)
  - [Trading Desk](#trading-desk)
  - [Admin Dashboard](#admin-dashboard)
- [Architecture](#architecture)
- [Use Cases](#use-cases)
- [Educational Value](#educational-value)
- [Testing](#testing)
- [Extensibility](#extensibility)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Why StockMart?

StockMart is a **production-ready multiplayer trading game platform** designed for event organizers, educational institutions, and gaming competitions. Host engaging trading battles with full control over every aspect of the game.

### Perfect For

| Audience | Why StockMart? |
|----------|---------------|
| **Event Organizers** | Turnkey solution for trading competitions at hackathons, gaming events, corporate team-building |
| **College Administrators** | Ready-to-deploy finance lab for economics, business, and finance courses |
| **Gaming Communities** | Unique multiplayer experience combining strategy, competition, and financial literacy |
| **Corporate Training** | Risk-free environment for teaching investment basics to employees |

### Key Selling Points

- **100+ Concurrent Players**: Tested architecture supporting large-scale competitions
- **Full Admin Control**: Start/stop markets, set time limits, configure starting capital, ban/mute users
- **Zero Setup for Players**: Students/players just open a browser—no downloads, no accounts to create externally
- **Real-time Everything**: Live charts, instant trade execution, dynamic leaderboards, global chat
- **Highly Configurable**: Customize starting capital, trading hours, circuit breakers, company roster
- **Self-Hosted**: Your data stays on your servers—perfect for institutional compliance

---

## For Event Organizers

Running a trading competition has never been easier. StockMart gives you complete control over your event.

### What You Can Do

| Capability | Description |
|------------|-------------|
| **One-Click Game Reset** | Initialize all players with equal starting capital instantly |
| **Market Hours Control** | Open/close trading at will—perfect for timed competitions |
| **Live Monitoring** | Watch trades execute in real-time, see who's winning |
| **Player Moderation** | Mute or ban disruptive participants |
| **Custom Companies** | Create fictional stocks or use preset market |
| **Circuit Breakers** | Auto-halt trading if prices move too fast |
| **Flexible Timing** | Run 15-minute speed rounds or multi-day tournaments |

### Sample Event Timeline

```
09:00  Admin opens registration (players create accounts)
09:30  Admin clicks "Initialize Game" → Everyone gets $100,000
09:35  Admin opens market → Trading begins!
10:30  Admin closes market → Final scores locked
10:35  Leaderboard displayed → Winners announced!
```

### Event Formats We Support

| Format | Duration | Best For |
|--------|----------|----------|
| **Lightning Round** | 15-30 min | Hackathons, lunch breaks |
| **Standard Competition** | 1-2 hours | Classroom exercises, meetups |
| **Marathon** | Full day | Major competitions, trading challenges |
| **Multi-Day Tournament** | Days/weeks | Semester-long courses, leagues |

---

## For College Administrators

StockMart is designed with educational institutions in mind. Deploy once, use for years.

### Deployment Benefits

| Feature | Benefit |
|---------|---------|
| **Self-Hosted** | Student data never leaves your network |
| **No Per-User Fees** | Unlimited students, one deployment |
| **Browser-Based** | Works on any device—no software to install |
| **Auto-Save** | Data persists across server restarts |
| **Low Requirements** | Runs on modest hardware (4GB RAM, 2 cores) |

### Integration with Curriculum

| Course | How StockMart Helps |
|--------|---------------------|
| **Introduction to Finance** | Hands-on experience with order types, portfolios |
| **Economics 101** | Demonstrate supply/demand, price discovery |
| **Investment Analysis** | Practice technical analysis on live charts |
| **Risk Management** | Teach short selling, margin, diversification |
| **Financial Markets** | Simulate market microstructure, order books |

### Running a Finance Lab

```bash
# Deploy on your campus server
cd backend && RUST_LOG=info cargo run --release

# Students access via browser
# http://your-server:5174

# Admin manages from dashboard
# Login as admin → Full control panel
```

### Multi-Section Support

Run multiple independent trading sessions:
1. Reset game between class sections
2. Each section competes on its own leaderboard
3. Export results for grading (JSON data files)

---

## Game Modes

| Mode | Duration | Players | Description |
|------|----------|---------|-------------|
| **Free Play** | Unlimited | Any | Open trading, build portfolios at your pace |
| **Speed Trading** | 15-30 min | 2-100+ | Fast-paced competition, quick decisions |
| **Tournament** | Multi-round | 8-64 | Elimination brackets, rising stakes |
| **Paper Trading** | Unlimited | Solo | Practice mode, doesn't affect leaderboard |
| **Team Battle** | 1-4 hours | Teams of 3-5 | Combined team net worth determines winner |
| **Classroom Mode** | 50-90 min | 20-40 | Perfect for class periods |

### How Competitions Work

```mermaid
sequenceDiagram
    participant Admin
    participant System
    participant Players

    Admin->>System: Initialize Game ($100K each)
    System->>Players: Portfolio reset notification
    Admin->>System: Open Market
    System->>Players: Trading enabled

    loop Trading Session
        Players->>System: Place orders
        System->>System: Match orders
        System->>Players: Real-time updates
    end

    Admin->>System: Close Market
    System->>Players: Trading disabled
    System->>Admin: Final leaderboard
```

---

## Configuration

StockMart is highly configurable to match your event needs.

### Game Configuration

Edit `backend/data/config.json`:

```json
{
    "default_starting_money": 10000000,
    "default_shares_per_company": 50,
    "circuit_breaker_threshold": 10,
    "circuit_breaker_window_seconds": 300,
    "auto_save_interval_seconds": 60
}
```

| Setting | Default | Description |
|---------|---------|-------------|
| `default_starting_money` | $100,000 | Virtual cash given to each player (in cents) |
| `default_shares_per_company` | 50 | Initial shares of each stock per player |
| `circuit_breaker_threshold` | 10% | Price change that triggers trading halt |
| `circuit_breaker_window_seconds` | 300 | Time window for circuit breaker |
| `auto_save_interval_seconds` | 60 | How often data saves to disk |

### Company Configuration

Customize the stock roster in `backend/data/companies.json`:

```json
[
    {
        "symbol": "TECH",
        "name": "TechCorp Industries",
        "current_price": 15000,
        "volatility": "high",
        "sector": "Technology"
    },
    {
        "symbol": "BANK",
        "name": "First National Bank",
        "current_price": 5000,
        "volatility": "low",
        "sector": "Finance"
    }
]
```

### Admin Controls (Runtime)

| Control | Effect |
|---------|--------|
| **Market Open/Close** | Enable/disable all trading instantly |
| **Initialize Game** | Reset all portfolios to starting values |
| **Mute User** | Prevent user from chatting |
| **Ban User** | Remove user from trading session |
| **Create Company (IPO)** | Add new stock mid-game |
| **Mark Bankrupt** | Remove company from trading |

---

## Scalability

StockMart is built for performance with Rust and async architecture.

### Performance Characteristics

| Metric | Capability |
|--------|------------|
| **Concurrent Users** | 100+ tested, designed for 500+ |
| **Orders/Second** | 1,000+ order matching throughput |
| **WebSocket Latency** | <50ms typical round-trip |
| **Memory Usage** | ~100MB base, ~1MB per 100 users |
| **Startup Time** | <2 seconds cold start |

### Architecture for Scale

```
┌─────────────────────────────────────────────────────────────┐
│                    Load Balancer (optional)                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 StockMart Backend (Rust)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  WebSocket  │  │  Matching   │  │   DashMap (Lock-    │  │
│  │   Handler   │──│   Engine    │──│   Free Concurrent)  │  │
│  │ (per-conn)  │  │ (async)     │  │      Storage        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│               Browser Clients (React + Zustand)              │
│         Player 1  │  Player 2  │  ...  │  Player N          │
└─────────────────────────────────────────────────────────────┘
```

### Why It Scales

| Technology | Benefit |
|------------|---------|
| **Rust** | Zero-cost abstractions, no garbage collection pauses |
| **Tokio** | Async runtime handling thousands of connections |
| **DashMap** | Lock-free concurrent HashMap for user/order data |
| **WebSocket** | Persistent connections, minimal overhead per message |
| **React 19** | Concurrent rendering, efficient DOM updates |
| **Zustand** | Minimal re-renders, fast state updates |

### Deployment Options

| Environment | Configuration |
|-------------|---------------|
| **Single Server** | 1 instance, handles 100+ users easily |
| **Docker** | `docker-compose up` for containerized deployment |
| **Cloud** | Deploy to AWS/GCP/Azure with load balancer |
| **Kubernetes** | Horizontal scaling for massive events |

---

## Quick Start

### Prerequisites

| Requirement | Version | Download |
|-------------|---------|----------|
| **Rust** | 1.76+ | [rustup.rs](https://rustup.rs/) |
| **Node.js** | 18+ | [nodejs.org](https://nodejs.org/) |

### Start Trading in 60 Seconds

```bash
# Clone the repository
git clone https://github.com/yourusername/stockmart.git
cd stockmart

# Start the backend (Terminal 1)
cd backend
RUST_LOG=info cargo run

# Start the frontend (Terminal 2)
cd frontend
npm install && npm run dev
```

Open [http://localhost:5174](http://localhost:5174), register an account, and start trading!

> **For detailed setup instructions**, see [docs/SETUP.md](./docs/SETUP.md)

---

## Features

### Trading Desk

The trader interface provides everything needed for a realistic trading experience:

<div align="center">
<img src="docs/screenshots/trading-desk.png" alt="Trading Desk Interface" width="800" />
</div>

| Feature | Description |
|---------|-------------|
| **Real-time Charts** | Professional candlestick charts powered by TradingView's Lightweight Charts |
| **Order Types** | Market orders (IOC) and Limit orders (GTC/IOC) |
| **Order Sides** | Buy, Sell, and Short selling with 150% margin requirement |
| **Order Book** | Live bid/ask depth with spread calculation |
| **Portfolio Tracking** | Real-time P&L, holdings value, and net worth |
| **Leaderboard** | Live rankings based on total net worth |
| **Global Chat** | Real-time communication between traders |
| **News Feed** | Simulated market news with sentiment indicators |
| **Market Indices** | Sector-based indices with live updates |

#### Symbol Selector

<div align="center">
<img src="docs/screenshots/trading-symbol-selector.png" alt="Symbol Selector" width="600" />
</div>

*Quickly switch between stocks with the searchable symbol dropdown*

---

### Admin Dashboard

Educators and game masters have complete control over the trading environment:

<div align="center">
<img src="docs/screenshots/admin-dashboard.png" alt="Admin Dashboard" width="800" />
</div>

| Feature | Description |
|---------|-------------|
| **Market Control** | Open/close trading with a single click |
| **Game Initialization** | Reset all portfolios with configurable starting capital |
| **Trader Management** | View all traders, mute chat, or ban users |
| **Company Management** | Create new stocks (IPOs), set volatility, mark bankrupt |
| **Circuit Breakers** | Configure automatic trading halts for volatility |
| **Real-time Monitoring** | Live stats, recent trades, and system health |

#### Game Control Panel

<div align="center">
<img src="docs/screenshots/admin-game-control.png" alt="Game Control" width="800" />
</div>

*Configure trading hours, circuit breakers, and initialize new game sessions*

#### Trader Management

<div align="center">
<img src="docs/screenshots/admin-traders.png" alt="Trader Management" width="800" />
</div>

*Monitor all traders with net worth, rankings, and moderation controls*

#### Company Management

<div align="center">
<img src="docs/screenshots/admin-companies.png" alt="Company Management" width="800" />
</div>

*Manage listed companies, adjust volatility, and monitor trading activity*

---

## Architecture

StockMart follows **Domain-Driven Design (DDD)** principles with a clean separation between backend and frontend:

```mermaid
graph TB
    subgraph Frontend["Frontend (React 19 + TypeScript)"]
        UI[Trading UI]
        Admin[Admin Dashboard]
        WS_Client[WebSocket Client]
        Store[Zustand Store]
    end

    subgraph Backend["Backend (Rust + Axum + Tokio)"]
        WS_Server[WebSocket Server]
        subgraph Services["Service Layer"]
            Engine[Matching Engine]
            Market[Market Service]
            LB[Leaderboard]
            News[News Generator]
            Chat[Chat Service]
            Persist[Persistence]
        end
        subgraph Domain["Domain Layer (DDD)"]
            User[User Context]
            Trading[Trading Context]
            MarketD[Market Context]
        end
        Repo[(In-Memory Store)]
    end

    UI --> Store
    Admin --> Store
    Store <--> WS_Client
    WS_Client <-->|"50+ Message Types"| WS_Server
    WS_Server --> Services
    Services --> Domain
    Domain --> Repo
    Persist -.->|"Auto-save (60s)"| Repo
```

### Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Backend Runtime** | Rust + Tokio | High-performance async runtime |
| **Web Framework** | Axum | Fast, ergonomic HTTP/WebSocket server |
| **Concurrency** | DashMap | Lock-free concurrent data structures |
| **Frontend Framework** | React 19 | Modern UI with concurrent features |
| **State Management** | Zustand | Lightweight, hooks-based state |
| **Styling** | TailwindCSS 4 | Utility-first CSS framework |
| **Charts** | Lightweight Charts | Professional financial charting |
| **E2E Testing** | Playwright | Cross-browser automated testing |

### Key Architectural Decisions

1. **WebSocket-First**: All real-time data flows through WebSocket with 50+ typed message types
2. **In-Memory Speed**: DashMap provides lock-free concurrent access for thousands of users
3. **Price-Time Priority**: Matching engine uses industry-standard order matching algorithm
4. **Auto-Persistence**: Data saved to JSON every 60 seconds with graceful shutdown
5. **Repository Pattern**: Easy to swap in-memory store for PostgreSQL in production

> **For detailed architecture documentation**, see [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)

---

## Use Cases

StockMart is designed for multiple audiences:

### For Gamers

| Use Case | Description |
|----------|-------------|
| **Competitive Trading** | Battle friends in real-time trading competitions |
| **Leaderboard Climbing** | Grind your way to the top of the rankings |
| **Strategy Gaming** | Use market analysis skills to outplay opponents |
| **Speed Challenges** | Quick 15-minute trading blitzes |
| **Risk-Free Betting** | Experience the thrill of trading without real losses |

### For Schools & Colleges

| Use Case | Description |
|----------|-------------|
| **Finance Courses** | Teach stock trading, portfolio management, and market dynamics |
| **Economics Labs** | Demonstrate supply/demand, price discovery, and market efficiency |
| **Trading Competitions** | Host class or inter-college trading challenges with leaderboards |
| **Capstone Projects** | Students can extend the platform with new features |

### For Learning Traders

| Use Case | Description |
|----------|-------------|
| **Risk-Free Practice** | Learn order types without risking real money |
| **Strategy Testing** | Experiment with different trading strategies |
| **Market Psychology** | Experience FOMO, fear, and greed in a safe environment |
| **Technical Analysis** | Practice reading candlestick charts and order books |

### For Engineers

| Use Case | Description |
|----------|-------------|
| **Learn Rust** | Production-grade async Rust with Tokio |
| **WebSocket Systems** | Build real-time applications with proper state management |
| **Domain-Driven Design** | Study bounded contexts and aggregate roots in practice |
| **Testing Patterns** | 599+ tests demonstrating unit, integration, and E2E patterns |

### For Product Managers

| Use Case | Description |
|----------|-------------|
| **Trading Platform UX** | Understand trading platform workflows |
| **Feature Simulation** | Prototype trading features before building |
| **Stakeholder Demos** | Demonstrate trading concepts to non-technical audiences |

---

## Educational Value

### What Students Learn

```
Trading Fundamentals          Technical Skills
├── Order Types               ├── Rust Programming
│   ├── Market Orders         ├── React/TypeScript
│   ├── Limit Orders          ├── WebSocket Protocol
│   └── Time-in-Force         ├── State Management
├── Trading Mechanics         ├── Testing (Unit/E2E)
│   ├── Bid/Ask Spread        └── Domain-Driven Design
│   ├── Order Book Depth
│   └── Price-Time Priority
├── Risk Management
│   ├── Portfolio Tracking
│   ├── Short Selling
│   └── Margin Requirements
└── Market Dynamics
    ├── Volatility
    ├── Circuit Breakers
    └── Market Indices
```

### Sample Lab Exercises

1. **Order Types Lab**: Place market and limit orders, observe execution differences
2. **Short Selling Lab**: Short a stock, understand margin requirements and risks
3. **Portfolio Lab**: Build a diversified portfolio, track P&L over time
4. **Market Making Lab**: Place bid/ask orders, learn about spread and liquidity
5. **Competition Lab**: 30-minute trading challenge with class leaderboard

---

## Testing

StockMart has comprehensive test coverage across backend and frontend:

### Backend Tests (Rust)

```
backend/tests/
├── auth_tests.rs           # Authentication & sessions
├── trading_tests.rs        # Order placement & matching
├── admin_tests.rs          # Admin operations
├── broadcast_tests.rs      # WebSocket broadcasting
├── concurrency_tests.rs    # Race conditions & thread safety
├── persistence_tests.rs    # Data saving & loading
├── security_tests.rs       # Input validation & auth
├── edge_case_tests.rs      # Boundary conditions
├── state_machine_tests.rs  # Order state transitions
├── validation_tests.rs     # Business rule validation
└── ... (16 test files total)
```

| Metric | Count |
|--------|-------|
| **Test Files** | 16 |
| **Test Cases** | 463 |
| **Lines of Test Code** | 13,444 |
| **Domain Coverage** | ~90% |
| **Service Coverage** | ~85% |
| **Overall Coverage** | ~85% |

```bash
# Run backend tests
cd backend
cargo test

# Run with output
cargo test -- --nocapture

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Frontend E2E Tests (Playwright)

```
frontend/tests/e2e/
├── auth/
│   ├── login.spec.ts
│   └── register.spec.ts
├── trading/
│   ├── order-placement.spec.ts
│   ├── portfolio.spec.ts
│   └── multi-user-trading.spec.ts
├── admin/
│   ├── dashboard.spec.ts
│   └── game-control.spec.ts
└── sync/
    └── state-sync.spec.ts
```

| Metric | Count |
|--------|-------|
| **Test Files** | 6 suites |
| **Test Cases** | 136 |
| **Lines of Test Code** | 3,604 |
| **Auth Flow Coverage** | 100% |
| **Trading Flow Coverage** | 95% |
| **Admin Flow Coverage** | 90% |

```bash
# Run E2E tests
cd frontend
npm test

# Run with UI
npm run test:ui

# Run specific suite
npm run test:trading
npm run test:admin
npm run test:auth
```

---

## Extensibility

StockMart is designed to be extended:

### Adding New Order Types

```rust
// backend/src/domain/trading/order.rs
pub enum OrderType {
    Market,
    Limit,
    // Add your new order type
    StopLoss { trigger_price: Price },
    TrailingStop { trail_percent: u8 },
}
```

### Adding Custom Indicators

```typescript
// frontend/src/features/trader/components/Chart.tsx
// Add technical indicators using Lightweight Charts API
chart.addLineSeries({
    color: '#2962FF',
    lineWidth: 2,
}).setData(calculateSMA(candles, 20));
```

### Building Trading Bots

```rust
// Use the WebSocket API to build algorithmic trading bots
// Connect to ws://localhost:3000/ws
// Send: { "type": "PlaceOrder", "symbol": "AAPL", ... }
// Receive: { "type": "OrderAck", ... }
```

### API Integration Points

| Endpoint | Purpose |
|----------|---------|
| `ws://localhost:3000/ws` | WebSocket for real-time trading |
| Messages: `PlaceOrder`, `CancelOrder` | Order management |
| Messages: `Subscribe`, `GetDepth` | Market data |
| Messages: `Chat`, `GetPortfolio` | User features |

---

## Roadmap

### Planned Features

| Feature | Status | Description |
|---------|--------|-------------|
| **PostgreSQL Persistence** | Planned | Replace JSON with proper database |
| **Options Trading** | Planned | Calls, puts, and options strategies |
| **Mobile App** | Planned | React Native trading app |
| **AI Trading Bots** | Planned | NPC traders for single-player practice |
| **REST API** | Planned | HTTP endpoints for integrations |
| **Historical Data** | Planned | Import real market data for backtesting |
| **Advanced Charts** | Planned | More technical indicators and drawing tools |

### Contributing Ideas

We welcome contributions! See [docs/CONTRIBUTING.md](./docs/CONTRIBUTING.md) for guidelines.

---

## Authentication

<div align="center">
<table>
<tr>
<td><img src="docs/screenshots/login-page.png" alt="Login Page" width="400" /></td>
<td><img src="docs/screenshots/register-page.png" alt="Register Page" width="400" /></td>
</tr>
<tr>
<td align="center"><em>Login Page</em></td>
<td align="center"><em>Registration Page</em></td>
</tr>
</table>
</div>

---

## Contributing

We love contributions! Whether it's:

- Reporting bugs
- Suggesting features
- Improving documentation
- Submitting pull requests

See [docs/CONTRIBUTING.md](./docs/CONTRIBUTING.md) for detailed guidelines.

### Quick Contribution Steps

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Commit (`git commit -m 'Add amazing feature'`)
5. Push (`git push origin feature/amazing-feature`)
6. Open a Pull Request

---

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**Built with love for educators, students, and trading enthusiasts**

[Report Bug](https://github.com/yourusername/stockmart/issues) · [Request Feature](https://github.com/yourusername/stockmart/issues) · [Documentation](./docs/)

</div>
