# StockMart - Real-time Virtual Stock Exchange

StockMart is a high-performance, real-time virtual stock exchange application built with **Rust** (Backend) and **React** (Frontend). It features a matching engine, live market data, portfolio management, global chat, and a leaderboard.

## 🚀 Tech Stack

### Backend
- **Rust**: Core language for performance and safety.
- **Axum**: High-performance web framework.
- **Tokio**: Asynchronous runtime.
- **DashMap**: Concurrent in-memory data storage.
- **Serde**: Serialization/Deserialization.
- **Tower-HTTP**: CORS and middleware.

### Frontend
- **React**: UI library.
- **Vite**: Build tool.
- **TypeScript**: Type safety.
- **Zustand**: State management.
- **TailwindCSS**: Styling.
- **Lightweight Charts**: Financial charting.
- **Lucide React**: Icons.

## ✨ Features

- **Real-time Trading**: Buy, Sell, and Short orders with a matching engine.
- **Live Market Data**: Real-time candlestick charts and order book updates via WebSocket.
- **Portfolio Management**: Track cash, holdings, and net worth in real-time.
- **Global Chat**: Real-time chat with other traders.
- **Leaderboard**: Live rankings based on total net worth.
- **Market News**: Simulated market news feed affecting sentiment.
- **Circuit Breakers**: Automatic trading halts for extreme volatility.
- **Persistence**: Auto-save/load of user and company data.

## 🛠️ Setup & Running

### Prerequisites
- **Rust** (latest stable)
- **Node.js** (v18+)

### Backend
1. Navigate to the backend directory:
   ```bash
   cd rust_stockmart/backend
   ```
2. Run the server:
   ```bash
   RUST_LOG=info cargo run
   ```
   The backend will start on `http://localhost:3000`.

### Frontend
1. Navigate to the frontend directory:
   ```bash
   cd rust_stockmart/frontend
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Start the development server:
   ```bash
   npm run dev
   ```
   The frontend will be available at `http://localhost:5173`.

## 📝 License
MIT
