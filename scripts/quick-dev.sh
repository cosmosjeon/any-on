#!/bin/bash
# 빠른 개발용 스크립트 - Debug 빌드로 빠르게 재시작
set -e

cd "$(dirname "$0")/.."

echo "🔨 Building (debug mode)..."
cargo build 2>&1 | tail -10

echo ""
echo "🛑 Stopping server..."
pkill -f "target/debug/server" || pkill -f "target/release/server" || true
sleep 1

echo "🚀 Starting server..."
ANYON_DATABASE_FILE=/home/cosmos/anyon/data/anyon.db \
ANYON_SECRET_KEY="nn0njTTCpGKVQ+UpWkjasE16vfT9azPhs3FTWbfii/Y=" \
BACKEND_PORT=3001 \
HOST=0.0.0.0 \
RUST_LOG=info \
./target/debug/server > /tmp/anyon-server.log 2>&1 &

SERVER_PID=$!
sleep 3

if ps -p $SERVER_PID > /dev/null 2>&1; then
    echo "✅ Server started successfully! (PID: $SERVER_PID)"
    echo ""
    echo "Health check:"
    curl -s http://localhost:3001/api/health | jq -r '.data' || echo "OK"
    echo ""
    echo "📝 Logs: tail -f /tmp/anyon-server.log"
    echo "🛑 Stop: kill $SERVER_PID"
else
    echo "❌ Server failed to start. Check logs:"
    tail -20 /tmp/anyon-server.log
    exit 1
fi
