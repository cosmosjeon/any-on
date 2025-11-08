#!/bin/bash

# Quick API Health Check and Port Discovery
# This script verifies the Anyon API is running and displays the base URL

set -e

echo "===== Anyon API Health Check ====="
echo ""

# Try to find the port from the port file
PORT_FILE="$HOME/.anyon/port"
if [ -f "$PORT_FILE" ]; then
  PORT=$(cat "$PORT_FILE")
  echo "Found port from ~/.anyon/port: ${PORT}"
else
  echo "Port file not found. Using default: 8080"
  PORT=8080
fi

BASE_URL="http://127.0.0.1:${PORT}"

echo "Testing API at: ${BASE_URL}"
echo ""

# Test health endpoint
echo "Checking /health endpoint..."
HEALTH_RESPONSE=$(curl -s "${BASE_URL}/health" 2>&1)

if echo "$HEALTH_RESPONSE" | grep -q "ok"; then
  echo "✓ API is healthy!"
  echo ""
  
  # Get system info
  echo "Getting system info..."
  INFO_RESPONSE=$(curl -s "${BASE_URL}/api/info")
  
  if echo "$INFO_RESPONSE" | grep -q '"success":true'; then
    echo "✓ API is accessible"
    
    # Extract some basic info
    ANALYTICS_ID=$(echo "$INFO_RESPONSE" | grep -o '"analytics_user_id":"[^"]*' | cut -d'"' -f4)
    OS_TYPE=$(echo "$INFO_RESPONSE" | grep -o '"os_type":"[^"]*' | cut -d'"' -f4)
    
    echo ""
    echo "System Information:"
    echo "  Analytics ID: ${ANALYTICS_ID}"
    echo "  OS Type: ${OS_TYPE}"
    
    # Check GitHub authentication
    echo ""
    echo "Checking GitHub authentication..."
    CHECK_RESPONSE=$(curl -s "${BASE_URL}/api/auth/github/check")
    TOKEN_STATUS=$(echo "$CHECK_RESPONSE" | grep -o '"data":"[^"]*' | cut -d'"' -f4)
    
    if [ "$TOKEN_STATUS" = "VALID" ]; then
      echo "✓ GitHub token is valid"
    else
      echo "⚠ GitHub token is invalid or not configured"
      echo "  Run: ./02-github-auth-flow.sh"
    fi
    
    echo ""
    echo "===== API Ready ====="
    echo ""
    echo "Base URL: ${BASE_URL}/api"
    echo ""
    echo "Quick commands:"
    echo "  List projects:  curl ${BASE_URL}/api/projects | jq ."
    echo "  System info:    curl ${BASE_URL}/api/info | jq ."
    echo ""
    
  else
    echo "⚠ API responded but returned an error:"
    echo "$INFO_RESPONSE" | jq . 2>/dev/null || echo "$INFO_RESPONSE"
  fi
  
else
  echo "✗ API health check failed"
  echo "Response: $HEALTH_RESPONSE"
  echo ""
  echo "Is the Anyon server running?"
  echo "Start with: pnpm run dev"
  exit 1
fi
