#!/bin/bash
# ============================================
# StockMart - Stop All Components
# ============================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PID_DIR="$PROJECT_DIR/.pids"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${RED}========================================${NC}"
echo -e "${RED}  StockMart - Stopping All Components${NC}"
echo -e "${RED}========================================${NC}"

# Function to stop a service by PID file
stop_service() {
    local name=$1
    local pid_file="$PID_DIR/$name.pid"

    echo -n "  Stopping $name..."

    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 $pid 2>/dev/null; then
            kill $pid 2>/dev/null
            sleep 1
            # Force kill if still running
            if kill -0 $pid 2>/dev/null; then
                kill -9 $pid 2>/dev/null
            fi
            echo -e " ${GREEN}Stopped (PID: $pid)${NC}"
        else
            echo -e " ${YELLOW}Not running${NC}"
        fi
        rm -f "$pid_file"
    else
        echo -e " ${YELLOW}No PID file found${NC}"
    fi
}

# Function to stop by port (fallback)
stop_by_port() {
    local port=$1
    local name=$2

    local pid=$(lsof -t -i :$port 2>/dev/null)
    if [ -n "$pid" ]; then
        echo -n "  Stopping $name on port $port..."
        kill $pid 2>/dev/null
        sleep 1
        if kill -0 $pid 2>/dev/null; then
            kill -9 $pid 2>/dev/null
        fi
        echo -e " ${GREEN}Stopped (PID: $pid)${NC}"
        return 0
    fi
    return 1
}

echo ""

# ============================================
# Stop Backend
# ============================================
echo -e "${YELLOW}[1/2] Backend${NC}"
stop_service "backend"
# Fallback: stop by port if PID file didn't work
if lsof -i :3000 > /dev/null 2>&1; then
    stop_by_port 3000 "Backend"
fi

# ============================================
# Stop Frontend
# ============================================
echo ""
echo -e "${YELLOW}[2/2] Frontend${NC}"
stop_service "frontend"
# Fallback: stop by port if PID file didn't work
if lsof -i :5174 > /dev/null 2>&1; then
    stop_by_port 5174 "Frontend"
fi

# Also kill any orphaned npm/node processes for this project
pkill -f "vite.*--port 5174" 2>/dev/null || true

# ============================================
# Summary
# ============================================
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  All Components Stopped!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Verify everything is stopped
echo "  Port Status:"
if lsof -i :3000 > /dev/null 2>&1; then
    echo -e "    Port 3000: ${RED}Still in use${NC}"
else
    echo -e "    Port 3000: ${GREEN}Free${NC}"
fi

if lsof -i :5174 > /dev/null 2>&1; then
    echo -e "    Port 5174: ${RED}Still in use${NC}"
else
    echo -e "    Port 5174: ${GREEN}Free${NC}"
fi
echo ""
