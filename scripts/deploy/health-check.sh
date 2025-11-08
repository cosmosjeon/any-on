#!/bin/bash

set -euo pipefail

VM_IP="${1:-}"
if [[ -z "$VM_IP" ]]; then
    echo "Usage: ./health-check.sh <VM_IP>" >&2
    exit 1
fi

BASE_URL="http://$VM_IP:3000"
ZONE="asia-northeast3-a"
INSTANCE_NAME="anyon"

echo "🏥 Running health checks for $BASE_URL..."
echo ""

# 1. 서버 응답 확인
echo "1️⃣ Checking server response..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL")
if [[ "$HTTP_CODE" == "200" ]]; then
    echo "   ✅ Server is responding (HTTP $HTTP_CODE)"
else
    echo "   ❌ Server error (HTTP $HTTP_CODE)"
    exit 1
fi

# 2. API 엔드포인트 확인
echo "2️⃣ Checking API endpoint..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/projects")
if [[ "$HTTP_CODE" == "200" ]]; then
    echo "   ✅ API is working (HTTP $HTTP_CODE)"
else
    echo "   ❌ API error (HTTP $HTTP_CODE)"
    exit 1
fi

# 3. WebSocket 포트 확인
echo "3️⃣ Checking WebSocket port..."
if nc -z "$VM_IP" 3000 >/dev/null 2>&1; then
    echo "   ✅ WebSocket port is open"
else
    echo "   ❌ WebSocket port is closed"
    exit 1
fi

# 4. Docker 확인
echo "4️⃣ Checking Docker on VM..."
DOCKER_STATUS=$(gcloud compute ssh "$INSTANCE_NAME" --zone="$ZONE" --command='systemctl is-active docker' 2>/dev/null || true)
if [[ "$DOCKER_STATUS" == "active" ]]; then
    echo "   ✅ Docker is running"
else
    echo "   ❌ Docker is not running"
    exit 1
fi

echo ""
echo "🎉 All health checks passed!"
echo "🌐 You can access Anyon at: $BASE_URL"
