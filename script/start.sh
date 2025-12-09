#!/bin/bash
# ============================================
# StockMart - Start All Components
# ============================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PID_DIR="$PROJECT_DIR/.pids"
LOG_DIR="$PROJECT_DIR/logs"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  StockMart - Starting All Components${NC}"
echo -e "${GREEN}========================================${NC}"

# Create directories
mkdir -p "$PID_DIR"
mkdir -p "$LOG_DIR"

# Function to check if a port is in use
check_port() {
    local port=$1
    if lsof -i :$port > /dev/null 2>&1; then
        return 0  # Port is in use
    else
        return 1  # Port is free
    fi
}

# Function to wait for a service to be ready
wait_for_service() {
    local port=$1
    local name=$2
    local max_attempts=30
    local attempt=1

    echo -n "  Waiting for $name to be ready..."
    while [ $attempt -le $max_attempts ]; do
        if check_port $port; then
            echo -e " ${GREEN}Ready!${NC}"
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done
    echo -e " ${RED}Timeout!${NC}"
    return 1
}

# ============================================
# Start Backend
# ============================================
echo ""
echo -e "${YELLOW}[1/2] Starting Backend...${NC}"

if check_port 3000; then
    echo -e "  ${YELLOW}Backend already running on port 3000${NC}"
else
    cd "$PROJECT_DIR/backend"

    # Build if needed
    if [ ! -f "target/release/stockmart-backend" ]; then
        echo "  Building backend (release mode)..."
        cargo build --release 2>&1 | tail -5
    fi

    # Start backend
    echo "  Starting backend server..."
    RUST_LOG=info nohup ./target/release/stockmart-backend > "$LOG_DIR/backend.log" 2>&1 &
    echo $! > "$PID_DIR/backend.pid"

    wait_for_service 3000 "Backend"
fi

# ============================================
# Start Frontend
# ============================================
echo ""
echo -e "${YELLOW}[2/2] Starting Frontend...${NC}"

if check_port 5174; then
    echo -e "  ${YELLOW}Frontend already running on port 5174${NC}"
else
    cd "$PROJECT_DIR/frontend"

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        echo "  Installing dependencies..."
        npm install > /dev/null 2>&1
    fi

    # Start frontend
    echo "  Starting frontend dev server..."
    nohup npm run dev -- --port 5174 > "$LOG_DIR/frontend.log" 2>&1 &
    echo $! > "$PID_DIR/frontend.pid"

    wait_for_service 5174 "Frontend"
fi

# ============================================
# Summary
# ============================================
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  All Components Started!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "  Services:"
echo -e "    Backend:  ${GREEN}http://localhost:3000${NC}"
echo -e "    Frontend: ${GREEN}http://localhost:5174${NC}"
echo ""
echo "  Logs:"
echo "    Backend:  $LOG_DIR/backend.log"
echo "    Frontend: $LOG_DIR/frontend.log"
echo ""
echo "  To stop all services, run: ./script/stop.sh"
echo ""
