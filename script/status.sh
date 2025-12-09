#!/bin/bash
# ============================================
# StockMart - Check Status of All Components
# ============================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PID_DIR="$PROJECT_DIR/.pids"
LOG_DIR="$PROJECT_DIR/logs"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  StockMart - Service Status${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Function to check service status
check_service() {
    local name=$1
    local port=$2
    local url=$3

    echo -e "${YELLOW}$name${NC}"

    # Check if port is in use
    local pid=$(lsof -t -i :$port 2>/dev/null | head -1)

    if [ -n "$pid" ]; then
        echo -e "  Status:  ${GREEN}Running${NC}"
        echo -e "  PID:     $pid"
        echo -e "  Port:    $port"
        echo -e "  URL:     $url"

        # Check if responding
        if curl -s --max-time 2 "$url" > /dev/null 2>&1; then
            echo -e "  Health:  ${GREEN}Responding${NC}"
        else
            echo -e "  Health:  ${YELLOW}Not responding (may be starting)${NC}"
        fi
    else
        echo -e "  Status:  ${RED}Stopped${NC}"
        echo -e "  Port:    $port (not in use)"
    fi
    echo ""
}

# ============================================
# Check Backend
# ============================================
check_service "Backend" 3000 "http://localhost:3000"

# ============================================
# Check Frontend
# ============================================
check_service "Frontend" 5174 "http://localhost:5174"

# ============================================
# Log Files
# ============================================
echo -e "${YELLOW}Log Files${NC}"
if [ -f "$LOG_DIR/backend.log" ]; then
    echo "  Backend:  $LOG_DIR/backend.log"
    echo "            (Last 3 lines):"
    tail -3 "$LOG_DIR/backend.log" 2>/dev/null | sed 's/^/            /'
else
    echo "  Backend:  No log file"
fi
echo ""

if [ -f "$LOG_DIR/frontend.log" ]; then
    echo "  Frontend: $LOG_DIR/frontend.log"
    echo "            (Last 3 lines):"
    tail -3 "$LOG_DIR/frontend.log" 2>/dev/null | sed 's/^/            /'
else
    echo "  Frontend: No log file"
fi
echo ""

echo -e "${BLUE}========================================${NC}"
echo ""
