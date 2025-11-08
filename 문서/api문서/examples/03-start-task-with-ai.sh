#!/bin/bash

# Example: Start a Task with AI Coding Agent
# This script creates a task and starts it with Claude Code

set -e

# Configuration
BASE_URL="http://127.0.0.1:8080/api"
PROJECT_ID="${1:-}"  # Pass project ID as first argument

if [ -z "$PROJECT_ID" ]; then
  echo "Usage: $0 <project-id>"
  echo ""
  echo "Example:"
  echo "  $0 123e4567-e89b-12d3-a456-426614174000"
  exit 1
fi

echo "===== Start Task with AI Coding Agent ====="
echo ""
echo "Project ID: ${PROJECT_ID}"
echo ""

# Get project branches
echo "Step 1: Getting available branches..."
BRANCHES_RESPONSE=$(curl -s "${BASE_URL}/projects/${PROJECT_ID}/branches")

if echo "$BRANCHES_RESPONSE" | grep -q '"success":true'; then
  echo "Available branches:"
  echo "$BRANCHES_RESPONSE" | jq -r '.data[] | "  - \(.name) \(if .is_current then "(current)" else "" end)"'

  # Get the current branch as default base
  BASE_BRANCH=$(echo "$BRANCHES_RESPONSE" | jq -r '.data[] | select(.is_current == true) | .name' | head -1)

  if [ -z "$BASE_BRANCH" ]; then
    BASE_BRANCH="main"
  fi

  echo ""
  echo "Using base branch: ${BASE_BRANCH}"
else
  echo "✗ Failed to get branches, using 'main' as default"
  BASE_BRANCH="main"
fi

echo ""

# Create and start task
TASK_TITLE="Implement user authentication"
TASK_DESCRIPTION="Add JWT-based authentication with the following requirements:
- Login endpoint with email/password
- Registration endpoint
- Token refresh mechanism
- Password hashing with bcrypt
- Input validation
- Error handling"

echo "Step 2: Creating task and starting AI agent..."
echo "  Title: ${TASK_TITLE}"
echo "  Executor: Claude Code"
echo "  Base branch: ${BASE_BRANCH}"
echo ""

CREATE_AND_START_RESPONSE=$(curl -s -X POST "${BASE_URL}/tasks/create-and-start" \
  -H "Content-Type: application/json" \
  -d "{
    \"task\": {
      \"project_id\": \"${PROJECT_ID}\",
      \"title\": \"${TASK_TITLE}\",
      \"description\": \"${TASK_DESCRIPTION}\",
      \"parent_task_attempt\": null,
      \"image_ids\": null
    },
    \"executor_profile_id\": {
      \"executor\": \"CLAUDE_CODE\",
      \"variant\": null
    },
    \"base_branch\": \"${BASE_BRANCH}\"
  }")

if echo "$CREATE_AND_START_RESPONSE" | grep -q '"success":true'; then
  TASK_ID=$(echo "$CREATE_AND_START_RESPONSE" | jq -r '.data.id')
  HAS_IN_PROGRESS=$(echo "$CREATE_AND_START_RESPONSE" | jq -r '.data.has_in_progress_attempt')

  echo "✓ Task created and started successfully!"
  echo "  Task ID: ${TASK_ID}"
  echo "  In Progress: ${HAS_IN_PROGRESS}"
  echo ""

  # Get task attempts
  echo "Step 3: Getting task attempts..."
  ATTEMPTS_RESPONSE=$(curl -s "${BASE_URL}/task-attempts?task_id=${TASK_ID}")

  if echo "$ATTEMPTS_RESPONSE" | grep -q '"success":true'; then
    ATTEMPT_ID=$(echo "$ATTEMPTS_RESPONSE" | jq -r '.data[0].id')
    BRANCH_NAME=$(echo "$ATTEMPTS_RESPONSE" | jq -r '.data[0].branch')

    echo "✓ Task attempt created:"
    echo "  Attempt ID: ${ATTEMPT_ID}"
    echo "  Branch: ${BRANCH_NAME}"
    echo ""

    # Get execution processes
    echo "Step 4: Getting execution processes..."
    sleep 2  # Give it a moment to start

    PROCESSES_RESPONSE=$(curl -s "${BASE_URL}/execution-processes?task_attempt_id=${ATTEMPT_ID}")

    if echo "$PROCESSES_RESPONSE" | grep -q '"success":true'; then
      echo "$PROCESSES_RESPONSE" | jq -r '.data[] | "  - Process \(.id)\n    Status: \(.status)\n    Started: \(.started_at)"'
      echo ""
    fi

    echo "===== Task Started Successfully ====="
    echo ""
    echo "Monitor progress:"
    echo "  - View in Anyon UI"
    echo "  - Stream logs: ws://${BASE_URL#http://}/execution-processes/<process-id>/normalized-logs/ws"
    echo "  - Get branch status: curl ${BASE_URL}/task-attempts/${ATTEMPT_ID}/branch-status"
    echo ""
    echo "Send follow-up instructions:"
    echo "  curl -X POST ${BASE_URL}/task-attempts/${ATTEMPT_ID}/follow-up \\"
    echo "    -H 'Content-Type: application/json' \\"
    echo "    -d '{\"prompt\": \"Add unit tests for the authentication endpoints\"}'"
    echo ""
  else
    echo "⚠ Task created but failed to get attempts:"
    echo "$ATTEMPTS_RESPONSE" | jq .
  fi
else
  echo "✗ Failed to create and start task:"
  echo "$CREATE_AND_START_RESPONSE" | jq .
  exit 1
fi
