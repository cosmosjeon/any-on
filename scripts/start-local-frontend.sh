#!/bin/bash
# Start Local Frontend Development Server
# 이 스크립트는 로컬 PC에서 프론트엔드 개발 서버를 시작합니다.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Anyon Frontend Development Server${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if .env exists
if [ ! -f ".env" ]; then
    echo -e "${RED}Error: .env file not found!${NC}"
    echo "Creating .env from .env.example..."
    if [ -f ".env.example" ]; then
        cp .env.example .env
        echo -e "${GREEN}✓ .env created from .env.example${NC}"
        echo -e "${YELLOW}Please edit .env and set BACKEND_HOST to your VM's IP or domain${NC}"
        exit 0
    else
        echo -e "${RED}Error: .env.example not found${NC}"
        exit 1
    fi
fi

# Load .env
echo -e "${YELLOW}Loading environment variables from .env...${NC}"
export $(cat .env | grep -v '^#' | xargs)

# Display configuration
echo -e "${GREEN}✓ Configuration loaded${NC}"
echo ""
echo "Frontend will run on: http://localhost:${FRONTEND_PORT:-3000}"
echo "Backend is expected at: http://${BACKEND_HOST:-localhost}:${BACKEND_PORT:-3001}"
echo ""

# Check if BACKEND_HOST looks like localhost
if [[ "$BACKEND_HOST" == "localhost" || "$BACKEND_HOST" == "127.0.0.1" ]]; then
    echo -e "${YELLOW}⚠️  Warning: BACKEND_HOST is set to localhost${NC}"
    echo "If you want to connect to a cloud backend, update BACKEND_HOST in .env"
    echo "Example: BACKEND_HOST=123.45.67.89"
    echo ""
fi

# Check if node_modules exists
if [ ! -d "frontend/node_modules" ]; then
    echo -e "${YELLOW}node_modules not found. Installing dependencies...${NC}"
    cd frontend && pnpm install && cd ..
    echo -e "${GREEN}✓ Dependencies installed${NC}"
    echo ""
fi

# Start frontend
echo -e "${GREEN}Starting frontend development server...${NC}"
echo "Press Ctrl+C to stop"
echo ""

cd frontend && pnpm run dev
