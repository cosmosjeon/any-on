#!/bin/bash

# Example: GitHub OAuth Device Flow Authentication
# This script demonstrates the complete GitHub authentication process

set -e

# Configuration
BASE_URL="http://127.0.0.1:8080/api"
POLL_INTERVAL=5  # seconds

echo "===== GitHub OAuth Device Flow Authentication ====="
echo ""

# Step 1: Start device flow
echo "Step 1: Starting GitHub device flow..."
START_RESPONSE=$(curl -s -X POST "${BASE_URL}/auth/github/device/start")

if echo "$START_RESPONSE" | grep -q '"success":true'; then
  USER_CODE=$(echo "$START_RESPONSE" | jq -r '.data.user_code')
  VERIFICATION_URI=$(echo "$START_RESPONSE" | jq -r '.data.verification_uri')
  EXPIRES_IN=$(echo "$START_RESPONSE" | jq -r '.data.expires_in')
  INTERVAL=$(echo "$START_RESPONSE" | jq -r '.data.interval')

  echo "✓ Device flow started successfully!"
  echo ""
  echo "╔════════════════════════════════════════════════════╗"
  echo "║  Please complete authentication:                   ║"
  echo "║                                                    ║"
  echo "║  1. Visit: ${VERIFICATION_URI}"
  echo "║  2. Enter code: ${USER_CODE}"
  echo "║                                                    ║"
  echo "║  Code expires in: ${EXPIRES_IN} seconds           ║"
  echo "╚════════════════════════════════════════════════════╝"
  echo ""
else
  echo "✗ Failed to start device flow:"
  echo "$START_RESPONSE" | jq .
  exit 1
fi

# Step 2: Poll for authorization
echo "Waiting for authorization (polling every ${INTERVAL} seconds)..."
echo "Press Ctrl+C to cancel"
echo ""

POLL_COUNT=0
MAX_POLLS=$((EXPIRES_IN / INTERVAL))

while [ $POLL_COUNT -lt $MAX_POLLS ]; do
  sleep $INTERVAL
  POLL_COUNT=$((POLL_COUNT + 1))

  POLL_RESPONSE=$(curl -s -X POST "${BASE_URL}/auth/github/device/poll")
  STATUS=$(echo "$POLL_RESPONSE" | jq -r '.data')

  case "$STATUS" in
    "SUCCESS")
      echo ""
      echo "✓ Authentication successful!"
      echo ""

      # Step 3: Verify token
      echo "Step 3: Verifying GitHub token..."
      CHECK_RESPONSE=$(curl -s "${BASE_URL}/auth/github/check")
      TOKEN_STATUS=$(echo "$CHECK_RESPONSE" | jq -r '.data')

      if [ "$TOKEN_STATUS" = "VALID" ]; then
        echo "✓ Token is valid and ready to use!"
      else
        echo "⚠ Token verification failed"
      fi

      echo ""
      echo "===== Authentication Complete ====="
      echo ""
      echo "You can now use GitHub features:"
      echo "- Push branches to GitHub"
      echo "- Create pull requests"
      echo "- Clone private repositories"
      exit 0
      ;;

    "SLOW_DOWN")
      echo "⚠ Polling too fast, slowing down..."
      INTERVAL=$((INTERVAL + 5))
      ;;

    "AUTHORIZATION_PENDING")
      echo -n "."
      ;;

    *)
      echo ""
      echo "✗ Unexpected response: $STATUS"
      echo "$POLL_RESPONSE" | jq .
      exit 1
      ;;
  esac
done

echo ""
echo "✗ Authentication timed out after ${EXPIRES_IN} seconds"
echo "Please run the script again to restart authentication"
exit 1
