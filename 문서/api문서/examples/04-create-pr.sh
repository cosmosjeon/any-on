#!/bin/bash

# Example: Create GitHub Pull Request
# This script pushes a task attempt branch and creates a PR

set -e

# Configuration
BASE_URL="http://127.0.0.1:8080/api"
ATTEMPT_ID="${1:-}"

if [ -z "$ATTEMPT_ID" ]; then
  echo "Usage: $0 <task-attempt-id>"
  echo ""
  echo "Example:"
  echo "  $0 123e4567-e89b-12d3-a456-426614174000"
  exit 1
fi

echo "===== Create GitHub Pull Request ====="
echo ""
echo "Task Attempt ID: ${ATTEMPT_ID}"
echo ""

# Step 1: Check GitHub token
echo "Step 1: Checking GitHub authentication..."
CHECK_RESPONSE=$(curl -s "${BASE_URL}/auth/github/check")
TOKEN_STATUS=$(echo "$CHECK_RESPONSE" | jq -r '.data')

if [ "$TOKEN_STATUS" != "VALID" ]; then
  echo "✗ GitHub token is invalid or not configured"
  echo "Please run: ./02-github-auth-flow.sh"
  exit 1
fi

echo "✓ GitHub token is valid"
echo ""

# Step 2: Get task attempt details
echo "Step 2: Getting task attempt details..."
ATTEMPT_RESPONSE=$(curl -s "${BASE_URL}/task-attempts/${ATTEMPT_ID}")

if ! echo "$ATTEMPT_RESPONSE" | grep -q '"success":true'; then
  echo "✗ Task attempt not found"
  echo "$ATTEMPT_RESPONSE" | jq .
  exit 1
fi

BRANCH_NAME=$(echo "$ATTEMPT_RESPONSE" | jq -r '.data.branch')
TARGET_BRANCH=$(echo "$ATTEMPT_RESPONSE" | jq -r '.data.target_branch')
TASK_ID=$(echo "$ATTEMPT_RESPONSE" | jq -r '.data.task_id')

echo "✓ Task attempt found:"
echo "  Branch: ${BRANCH_NAME}"
echo "  Target: ${TARGET_BRANCH}"
echo ""

# Step 3: Get task details for PR title/body
echo "Step 3: Getting task details..."
TASK_RESPONSE=$(curl -s "${BASE_URL}/tasks/${TASK_ID}")

if echo "$TASK_RESPONSE" | grep -q '"success":true'; then
  TASK_TITLE=$(echo "$TASK_RESPONSE" | jq -r '.data.title')
  TASK_DESCRIPTION=$(echo "$TASK_RESPONSE" | jq -r '.data.description // ""')

  echo "✓ Task details:"
  echo "  Title: ${TASK_TITLE}"
else
  echo "⚠ Could not get task details, using defaults"
  TASK_TITLE="Task changes"
  TASK_DESCRIPTION=""
fi

echo ""

# Step 4: Get branch status
echo "Step 4: Checking branch status..."
STATUS_RESPONSE=$(curl -s "${BASE_URL}/task-attempts/${ATTEMPT_ID}/branch-status")

if echo "$STATUS_RESPONSE" | grep -q '"success":true'; then
  COMMITS_AHEAD=$(echo "$STATUS_RESPONSE" | jq -r '.data.commits_ahead')
  HAS_UNCOMMITTED=$(echo "$STATUS_RESPONSE" | jq -r '.data.has_uncommitted_changes')

  echo "  Commits ahead: ${COMMITS_AHEAD}"
  echo "  Uncommitted changes: ${HAS_UNCOMMITTED}"

  if [ "$COMMITS_AHEAD" = "0" ] || [ "$COMMITS_AHEAD" = "null" ]; then
    echo ""
    echo "⚠ No commits to push. Make changes first."
    exit 1
  fi

  if [ "$HAS_UNCOMMITTED" = "true" ]; then
    echo ""
    echo "⚠ Warning: You have uncommitted changes"
    echo "These will not be included in the PR"
  fi
else
  echo "⚠ Could not check branch status"
fi

echo ""

# Step 5: Push branch to GitHub
echo "Step 5: Pushing branch to GitHub..."
PUSH_RESPONSE=$(curl -s -X POST "${BASE_URL}/task-attempts/${ATTEMPT_ID}/push")

if echo "$PUSH_RESPONSE" | grep -q '"success":true'; then
  echo "✓ Branch pushed successfully"
else
  echo "✗ Failed to push branch:"
  echo "$PUSH_RESPONSE" | jq .
  exit 1
fi

echo ""

# Step 6: Create pull request
echo "Step 6: Creating pull request..."

# Construct PR body
PR_BODY="## Task Description

${TASK_DESCRIPTION}

## Changes

This PR implements the changes described in the task.

## Testing

- [ ] Tested locally
- [ ] All tests pass
- [ ] No linting errors

---
Created by Anyon
Task ID: ${TASK_ID}
Attempt ID: ${ATTEMPT_ID}"

PR_RESPONSE=$(curl -s -X POST "${BASE_URL}/task-attempts/${ATTEMPT_ID}/pr" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"${TASK_TITLE}\",
    \"body\": $(echo "$PR_BODY" | jq -Rs .),
    \"target_branch\": null
  }")

if echo "$PR_RESPONSE" | grep -q '"success":true'; then
  PR_URL=$(echo "$PR_RESPONSE" | jq -r '.data')

  echo "✓ Pull request created successfully!"
  echo ""
  echo "╔════════════════════════════════════════════════════╗"
  echo "║  PR URL: ${PR_URL}"
  echo "╚════════════════════════════════════════════════════╝"
  echo ""
  echo "The PR has been opened in your browser."
  echo ""

  # Step 7: Verify PR was attached
  echo "Step 7: Verifying PR attachment..."
  STATUS_RESPONSE=$(curl -s "${BASE_URL}/task-attempts/${ATTEMPT_ID}/branch-status")

  if echo "$STATUS_RESPONSE" | grep -q '"merges"'; then
    echo "✓ PR successfully attached to task attempt"
  fi

  echo ""
  echo "===== PR Created Successfully ====="

else
  # Check if it's a GitHub-specific error
  ERROR_DATA=$(echo "$PR_RESPONSE" | jq -r '.error_data')

  if [ "$ERROR_DATA" != "null" ]; then
    echo "✗ GitHub API error: ${ERROR_DATA}"
  else
    echo "✗ Failed to create pull request:"
    echo "$PR_RESPONSE" | jq .
  fi

  exit 1
fi
