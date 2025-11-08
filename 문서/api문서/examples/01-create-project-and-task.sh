#!/bin/bash

# Example: Create a project and task in Anyon
# This script demonstrates the basic workflow of creating a project and starting a task

set -e  # Exit on error

# Configuration
BASE_URL="http://127.0.0.1:8080/api"  # Update port as needed
PROJECT_NAME="My Demo Project"
PROJECT_PATH="/Users/$(whoami)/projects/demo-project"
TASK_TITLE="Add dark mode support"
TASK_DESCRIPTION="Implement a dark mode theme switcher for the application"

echo "===== Anyon API Demo: Create Project and Task ====="
echo ""

# Step 1: Create a new project
echo "Step 1: Creating project '${PROJECT_NAME}'..."
CREATE_PROJECT_RESPONSE=$(curl -s -X POST "${BASE_URL}/projects" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"${PROJECT_NAME}\",
    \"git_repo_path\": \"${PROJECT_PATH}\",
    \"use_existing_repo\": false,
    \"setup_script\": \"npm install\",
    \"dev_script\": \"npm run dev\",
    \"cleanup_script\": null,
    \"copy_files\": null
  }")

# Check if project creation was successful
if echo "$CREATE_PROJECT_RESPONSE" | grep -q '"success":true'; then
  PROJECT_ID=$(echo "$CREATE_PROJECT_RESPONSE" | grep -o '"id":"[^"]*' | head -1 | cut -d'"' -f4)
  echo "✓ Project created successfully!"
  echo "  Project ID: ${PROJECT_ID}"
else
  echo "✗ Failed to create project:"
  echo "$CREATE_PROJECT_RESPONSE" | jq .
  exit 1
fi

echo ""

# Step 2: Create a task in the project
echo "Step 2: Creating task '${TASK_TITLE}'..."
CREATE_TASK_RESPONSE=$(curl -s -X POST "${BASE_URL}/tasks" \
  -H "Content-Type: application/json" \
  -d "{
    \"project_id\": \"${PROJECT_ID}\",
    \"title\": \"${TASK_TITLE}\",
    \"description\": \"${TASK_DESCRIPTION}\",
    \"parent_task_attempt\": null,
    \"image_ids\": null
  }")

# Check if task creation was successful
if echo "$CREATE_TASK_RESPONSE" | grep -q '"success":true'; then
  TASK_ID=$(echo "$CREATE_TASK_RESPONSE" | grep -o '"id":"[^"]*' | head -1 | cut -d'"' -f4)
  echo "✓ Task created successfully!"
  echo "  Task ID: ${TASK_ID}"
else
  echo "✗ Failed to create task:"
  echo "$CREATE_TASK_RESPONSE" | jq .
  exit 1
fi

echo ""

# Step 3: List all projects (verification)
echo "Step 3: Listing all projects..."
curl -s "${BASE_URL}/projects" | jq '.data[] | {id, name, git_repo_path}'

echo ""

# Step 4: List tasks for the project (verification)
echo "Step 4: Listing tasks for project..."
curl -s "${BASE_URL}/tasks?project_id=${PROJECT_ID}" | jq '.data[] | {id, title, status}'

echo ""
echo "===== Demo Complete ====="
echo ""
echo "Next steps:"
echo "1. Start a task attempt:"
echo "   curl -X POST ${BASE_URL}/tasks/create-and-start \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{...}'"
echo ""
echo "2. View the project in Anyon UI"
echo "3. Monitor execution processes"
