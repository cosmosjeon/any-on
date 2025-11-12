#!/bin/bash
# Anyon Cloud Deployment Script
# 이 스크립트는 클라우드 VM에서 실행하여 백엔드를 배포합니다.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Anyon Cloud Backend Deployment${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if .env.cloud exists
if [ ! -f ".env.cloud" ]; then
    echo -e "${RED}Error: .env.cloud file not found!${NC}"
    echo "Please create .env.cloud file with required environment variables."
    echo "See CLOUD_DEPLOYMENT_GUIDE.md for details."
    exit 1
fi

# Load .env.cloud
echo -e "${YELLOW}Loading environment variables from .env.cloud...${NC}"
export $(cat .env.cloud | grep -v '^#' | xargs)

# Check HOST setting
if [ "$HOST" != "0.0.0.0" ]; then
    echo -e "${YELLOW}Warning: HOST is set to '$HOST'${NC}"
    echo -e "${YELLOW}For external access, HOST should be '0.0.0.0'${NC}"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust is not installed!${NC}"
    echo "Please install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo -e "${GREEN}✓ Environment variables loaded${NC}"
echo ""

# Build with cloud feature
echo -e "${YELLOW}Building backend with cloud feature...${NC}"
echo "This may take several minutes..."
cargo build --release --features cloud

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Build successful${NC}"
echo ""

# Create necessary directories
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p "${ANYON_ASSET_DIR:-/home/user/anyon/data}"
mkdir -p "${ANYON_TEMP_DIR:-/home/user/anyon/tmp}"
mkdir -p "${ANYON_WORKTREE_DIR:-/home/user/anyon/worktrees}"
mkdir -p "${ANYON_WORKSPACE_DIR:-/home/user/anyon/workspace}"

# Create log directory
LOG_DIR=$(dirname "${ANYON_LOG_FILE:-/home/user/anyon/logs/server.log}")
mkdir -p "$LOG_DIR"

echo -e "${GREEN}✓ Directories created${NC}"
echo ""

# Check if server is already running
if pgrep -f "target/release/server" > /dev/null; then
    echo -e "${YELLOW}Server is already running. Stopping...${NC}"
    pkill -f "target/release/server" || true
    sleep 2
fi

# Ask user how to run the server
echo -e "${YELLOW}How would you like to run the server?${NC}"
echo "1) Foreground (you'll see logs in terminal)"
echo "2) Background with nohup"
echo "3) Install as systemd service (recommended for production)"
echo ""
read -p "Choose option (1-3): " -n 1 -r
echo ""

case $REPLY in
    1)
        echo -e "${GREEN}Starting server in foreground...${NC}"
        echo "Press Ctrl+C to stop"
        echo ""
        ./target/release/server
        ;;
    2)
        echo -e "${GREEN}Starting server in background...${NC}"
        LOG_FILE="${ANYON_LOG_FILE:-./server.log}"
        nohup ./target/release/server > "$LOG_FILE" 2>&1 &
        sleep 2
        if pgrep -f "target/release/server" > /dev/null; then
            echo -e "${GREEN}✓ Server started successfully${NC}"
            echo "Server is running on http://${HOST}:${BACKEND_PORT}"
            echo "Log file: $LOG_FILE"
            echo ""
            echo "To view logs: tail -f $LOG_FILE"
            echo "To stop: pkill -f 'target/release/server'"
        else
            echo -e "${RED}Failed to start server. Check logs: $LOG_FILE${NC}"
            exit 1
        fi
        ;;
    3)
        echo -e "${GREEN}Installing systemd service...${NC}"

        # Check if systemd is available
        if ! command -v systemctl &> /dev/null; then
            echo -e "${RED}Error: systemd is not available on this system${NC}"
            exit 1
        fi

        # Create systemd service file
        SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
        PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

        sudo tee /etc/systemd/system/anyon-backend.service > /dev/null <<EOF
[Unit]
Description=Anyon Backend Server
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$PROJECT_DIR
EnvironmentFile=$PROJECT_DIR/.env.cloud
ExecStart=$PROJECT_DIR/target/release/server
Restart=on-failure
RestartSec=10
StandardOutput=append:${ANYON_LOG_FILE:-/home/user/anyon/logs/server.log}
StandardError=append:${ANYON_LOG_FILE:-/home/user/anyon/logs/server.log}

[Install]
WantedBy=multi-user.target
EOF

        # Reload systemd and start service
        sudo systemctl daemon-reload
        sudo systemctl enable anyon-backend
        sudo systemctl start anyon-backend

        sleep 2

        if sudo systemctl is-active --quiet anyon-backend; then
            echo -e "${GREEN}✓ Service installed and started successfully${NC}"
            echo "Server is running on http://${HOST}:${BACKEND_PORT}"
            echo ""
            echo "Useful commands:"
            echo "  sudo systemctl status anyon-backend    # Check status"
            echo "  sudo systemctl stop anyon-backend      # Stop service"
            echo "  sudo systemctl restart anyon-backend   # Restart service"
            echo "  sudo journalctl -u anyon-backend -f    # View logs"
        else
            echo -e "${RED}Failed to start service${NC}"
            echo "Check status: sudo systemctl status anyon-backend"
            echo "Check logs: sudo journalctl -u anyon-backend"
            exit 1
        fi
        ;;
    *)
        echo -e "${RED}Invalid option${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Deployment Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Next steps:"
echo "1. Test the health endpoint: curl http://${HOST}:${BACKEND_PORT}/health"
echo "2. Configure your local frontend with BACKEND_HOST=${HOST}"
echo "3. Ensure firewall allows port ${BACKEND_PORT}"
echo ""
