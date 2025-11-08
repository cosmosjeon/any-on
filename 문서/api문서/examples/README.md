# API Examples

This directory contains practical examples demonstrating common Anyon API workflows.

## Prerequisites

All scripts require:
- `curl` - HTTP client
- `jq` - JSON processor (install: `brew install jq` on macOS)
- Running Anyon server (update `BASE_URL` in scripts if needed)

## Available Examples

### 1. Create Project and Task
**File:** `01-create-project-and-task.sh`

Demonstrates:
- Creating a new project
- Creating a task within the project
- Listing projects and tasks

```bash
./01-create-project-and-task.sh
```

### 2. GitHub Authentication Flow
**File:** `02-github-auth-flow.sh`

Demonstrates:
- Starting GitHub OAuth device flow
- Polling for user authorization
- Verifying token validity

```bash
./02-github-auth-flow.sh
```

### 3. Start Task with AI Agent
**File:** `03-start-task-with-ai.sh`

Demonstrates:
- Creating a task with detailed description
- Starting a task attempt with Claude Code
- Monitoring execution processes

```bash
./03-start-task-with-ai.sh <project-id>
```

### 4. Create GitHub Pull Request
**File:** `04-create-pr.sh`

Demonstrates:
- Checking branch status
- Pushing branch to GitHub
- Creating a pull request
- Attaching PR to task attempt

```bash
./04-create-pr.sh <task-attempt-id>
```

## Making Scripts Executable

```bash
chmod +x *.sh
```

## Complete Workflow Example

Here's a complete workflow using all scripts:

```bash
# 1. Create a project and task
./01-create-project-and-task.sh
# Note the PROJECT_ID and TASK_ID from output

# 2. Authenticate with GitHub (if not already)
./02-github-auth-flow.sh

# 3. Start the task with AI
./03-start-task-with-ai.sh <PROJECT_ID>
# Note the TASK_ATTEMPT_ID from output

# 4. Wait for AI to complete work (monitor in UI)

# 5. Create a PR
./04-create-pr.sh <TASK_ATTEMPT_ID>
```

## Additional Examples

### List All Projects

```bash
curl http://127.0.0.1:8080/api/projects | jq .
```

### Get Task Status

```bash
TASK_ID="your-task-id"
curl "http://127.0.0.1:8080/api/tasks/${TASK_ID}" | jq .
```

### Send Follow-up to AI

```bash
ATTEMPT_ID="your-attempt-id"
curl -X POST "http://127.0.0.1:8080/api/task-attempts/${ATTEMPT_ID}/follow-up" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Please add unit tests for all functions",
    "variant": null,
    "image_ids": null
  }' | jq .
```

### Get Branch Status

```bash
ATTEMPT_ID="your-attempt-id"
curl "http://127.0.0.1:8080/api/task-attempts/${ATTEMPT_ID}/branch-status" | jq .
```

### Upload Image

```bash
curl -X POST http://127.0.0.1:8080/api/images/upload \
  -F "image=@screenshot.png" | jq .
```

### Search Project Files

```bash
PROJECT_ID="your-project-id"
curl "http://127.0.0.1:8080/api/projects/${PROJECT_ID}/search?q=index.ts&mode=TaskForm" | jq .
```

### Stop Running Task

```bash
ATTEMPT_ID="your-attempt-id"
curl -X POST "http://127.0.0.1:8080/api/task-attempts/${ATTEMPT_ID}/stop" | jq .
```

### Rebase on Target Branch

```bash
ATTEMPT_ID="your-attempt-id"
curl -X POST "http://127.0.0.1:8080/api/task-attempts/${ATTEMPT_ID}/rebase" \
  -H "Content-Type: application/json" \
  -d '{
    "old_base_branch": null,
    "new_base_branch": "main"
  }' | jq .
```

### Merge to Target Branch

```bash
ATTEMPT_ID="your-attempt-id"
curl -X POST "http://127.0.0.1:8080/api/task-attempts/${ATTEMPT_ID}/merge" | jq .
```

## WebSocket Examples

### Stream Task Updates

```javascript
const ws = new WebSocket('ws://127.0.0.1:8080/api/tasks/stream/ws?project_id=<project-id>');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Task update:', data);
};
```

### Stream Execution Logs

```javascript
const ws = new WebSocket('ws://127.0.0.1:8080/api/execution-processes/<process-id>/normalized-logs/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Log entry:', data);
};
```

### Stream Git Diff

```javascript
const ws = new WebSocket('ws://127.0.0.1:8080/api/task-attempts/<attempt-id>/diff/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Diff update:', data);
};
```

## Python Examples

### Create Project

```python
import requests

base_url = "http://127.0.0.1:8080/api"

response = requests.post(f"{base_url}/projects", json={
    "name": "Python Project",
    "git_repo_path": "/Users/me/projects/python-demo",
    "use_existing_repo": False,
    "setup_script": "pip install -r requirements.txt",
    "dev_script": "python manage.py runserver",
    "cleanup_script": None,
    "copy_files": None
})

project = response.json()
print(f"Created project: {project['data']['id']}")
```

### Start Task with AI

```python
import requests

base_url = "http://127.0.0.1:8080/api"
project_id = "your-project-id"

response = requests.post(f"{base_url}/tasks/create-and-start", json={
    "task": {
        "project_id": project_id,
        "title": "Add logging",
        "description": "Implement structured logging with loguru",
        "parent_task_attempt": None,
        "image_ids": None
    },
    "executor_profile_id": {
        "executor": "CLAUDE_CODE",
        "variant": None
    },
    "base_branch": "main"
})

result = response.json()
print(f"Task started: {result['data']['id']}")
```

## TypeScript Examples

### Create Task (Type-Safe)

```typescript
import type { CreateTask, ApiResponse, Task } from '../shared/types';

async function createTask(projectId: string): Promise<Task> {
  const payload: CreateTask = {
    project_id: projectId,
    title: 'Implement feature X',
    description: 'Add support for feature X',
    parent_task_attempt: null,
    image_ids: null
  };

  const response = await fetch('http://127.0.0.1:8080/api/tasks', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload)
  });

  const result: ApiResponse<Task> = await response.json();

  if (!result.success) {
    throw new Error(result.message || 'Failed to create task');
  }

  return result.data!;
}
```

## Troubleshooting

### Port Issues

If scripts fail with connection errors:

1. Check Anyon server is running
2. Find actual port: `cat ~/.anyon/port`
3. Update `BASE_URL` in scripts

### Authentication Issues

If GitHub operations fail:

1. Check token: `curl http://127.0.0.1:8080/api/auth/github/check`
2. Re-authenticate: `./02-github-auth-flow.sh`

### jq Not Found

Install jq:
- macOS: `brew install jq`
- Ubuntu: `sudo apt-get install jq`
- Windows: Download from https://stedolan.github.io/jq/

## Contributing

Feel free to add more examples! Follow the existing patterns:
- Use bash scripts for CLI examples
- Include error handling
- Add clear comments
- Show example output
