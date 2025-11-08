# API Endpoints Reference

Complete reference for all Anyon API endpoints.

## Table of Contents

- [Health](#health)
- [System & Configuration](#system--configuration)
- [Authentication](#authentication)
- [Projects](#projects)
- [Tasks](#tasks)
- [Task Attempts](#task-attempts)
- [Execution Processes](#execution-processes)
- [Images](#images)
- [Events (SSE/WebSocket)](#events-sseworksocket)
- [Filesystem](#filesystem)
- [Containers](#containers)
- [Drafts](#drafts)
- [Tags](#tags)
- [Approvals](#approvals)

---

## Health

### Check API Health

**Endpoint:** `GET /health`

**Description:** Health check endpoint to verify the API is running.

**Response:**
```json
{
  "status": "ok"
}
```

**Example:**
```bash
curl http://127.0.0.1:PORT/health
```

---

## System & Configuration

### Get User System Info

**Endpoint:** `GET /api/info`

**Description:** Get complete system information including config, executor profiles, and capabilities.

**Response:**
```typescript
{
  success: true,
  data: {
    config: Config,
    analytics_user_id: string,
    executors: { [key in BaseCodingAgent]?: ExecutorConfig },
    environment: {
      os_type: string,
      os_version: string,
      os_architecture: string,
      bitness: string
    },
    capabilities: { [key: string]: Array<BaseAgentCapability> }
  }
}
```

**Example:**
```bash
curl http://127.0.0.1:PORT/api/info
```

### Update Configuration

**Endpoint:** `PUT /api/config`

**Description:** Update user configuration (theme, executor profile, GitHub settings, etc.)

**Request Body:**
```json
{
  "config_version": "1.0.0",
  "theme": "DARK",
  "executor_profile": {
    "executor": "CLAUDE_CODE",
    "variant": null
  },
  "disclaimer_acknowledged": true,
  "onboarding_acknowledged": true,
  "github_login_acknowledged": true,
  "telemetry_acknowledged": true,
  "analytics_enabled": true,
  "workspace_dir": "/path/to/workspace",
  "git_branch_prefix": "anyon",
  "language": "EN",
  "notifications": {
    "sound_enabled": true,
    "push_enabled": false,
    "sound_file": "ABSTRACT_SOUND1"
  },
  "editor": {
    "editor_type": "VS_CODE",
    "custom_command": null,
    "remote_ssh_host": null,
    "remote_ssh_user": null
  },
  "github": {
    "pat": null,
    "oauth_token": "ghp_xxx",
    "username": "myusername",
    "primary_email": "user@example.com",
    "default_pr_base": "main"
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": { /* Updated config */ }
}
```

### Get MCP Servers

**Endpoint:** `GET /api/mcp-config?executor=CLAUDE_CODE`

**Description:** Get Model Context Protocol server configuration for an executor.

**Query Parameters:**
- `executor` (required): BaseCodingAgent enum value

**Response:**
```json
{
  "success": true,
  "data": {
    "mcp_config": {
      "servers": { /* server definitions */ },
      "servers_path": ["mcpServers"],
      "template": {},
      "preconfigured": {},
      "is_toml_config": false
    },
    "config_path": "/path/to/config.json"
  }
}
```

### Update MCP Servers

**Endpoint:** `POST /api/mcp-config?executor=CLAUDE_CODE`

**Description:** Update MCP server configuration for an executor.

**Request Body:**
```json
{
  "servers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed"]
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxx"
      }
    }
  }
}
```

### Get Executor Profiles

**Endpoint:** `GET /api/profiles`

**Description:** Get all executor profile configurations as JSON.

**Response:**
```json
{
  "success": true,
  "data": {
    "content": "{ /* JSON string of profiles */ }",
    "path": "/path/to/profiles.json"
  }
}
```

### Update Executor Profiles

**Endpoint:** `PUT /api/profiles`

**Description:** Update executor profile configurations.

**Request Body:** Raw JSON string of ExecutorConfigs

**Example:**
```bash
curl -X PUT http://127.0.0.1:PORT/api/profiles \
  -H "Content-Type: application/json" \
  -d '{
    "executors": {
      "CLAUDE_CODE": {
        "default": {
          "CLAUDE_CODE": {
            "append_prompt": null,
            "model": "claude-sonnet-4-5",
            "approvals": true
          }
        }
      }
    }
  }'
```

### Get Sound File

**Endpoint:** `GET /api/sounds/{sound}`

**Description:** Get notification sound file.

**Path Parameters:**
- `sound`: SoundFile enum value (e.g., "ABSTRACT_SOUND1")

**Response:** WAV audio file

---

## Authentication

### Start GitHub Device Flow

**Endpoint:** `POST /api/auth/github/device/start`

**Description:** Initialize GitHub OAuth device flow.

**Response:**
```json
{
  "success": true,
  "data": {
    "user_code": "ABCD-1234",
    "verification_uri": "https://github.com/login/device",
    "expires_in": 899,
    "interval": 5
  }
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:PORT/api/auth/github/device/start
```

### Poll Device Flow

**Endpoint:** `POST /api/auth/github/device/poll`

**Description:** Poll for device flow authorization completion.

**Response:**
```json
{
  "success": true,
  "data": "SUCCESS" | "AUTHORIZATION_PENDING" | "SLOW_DOWN"
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:PORT/api/auth/github/device/poll
```

### Check GitHub Token

**Endpoint:** `GET /api/auth/github/check`

**Description:** Verify if stored GitHub token is valid.

**Response:**
```json
{
  "success": true,
  "data": "VALID" | "INVALID"
}
```

---

## Projects

### List Projects

**Endpoint:** `GET /api/projects`

**Description:** Get all projects.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "name": "My Project",
      "git_repo_path": "/path/to/repo",
      "setup_script": "npm install",
      "dev_script": "npm run dev",
      "cleanup_script": null,
      "copy_files": null,
      "created_at": "2025-01-08T00:00:00Z",
      "updated_at": "2025-01-08T00:00:00Z"
    }
  ]
}
```

**Example:**
```bash
curl http://127.0.0.1:PORT/api/projects
```

### Get Project

**Endpoint:** `GET /api/projects/{id}`

**Description:** Get a specific project by ID.

**Path Parameters:**
- `id` (required): Project UUID

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "My Project",
    "git_repo_path": "/path/to/repo",
    "setup_script": "npm install",
    "dev_script": "npm run dev",
    "cleanup_script": null,
    "copy_files": null,
    "created_at": "2025-01-08T00:00:00Z",
    "updated_at": "2025-01-08T00:00:00Z"
  }
}
```

### Create Project

**Endpoint:** `POST /api/projects`

**Description:** Create a new project.

**Request Body:**
```json
{
  "name": "My New Project",
  "git_repo_path": "/Users/me/projects/my-project",
  "use_existing_repo": false,
  "setup_script": "npm install && npm run build",
  "dev_script": "npm run dev",
  "cleanup_script": "npm run clean",
  "copy_files": null
}
```

**Field Details:**
- `name` (required): Project name
- `git_repo_path` (required): Absolute path to git repository
- `use_existing_repo` (required): If true, use existing repo; if false, initialize new repo
- `setup_script` (optional): Script to run when setting up task attempt
- `dev_script` (optional): Script to run development server
- `cleanup_script` (optional): Script to run after task completion
- `copy_files` (optional): Files to copy from main repo to worktree

**Response:**
```json
{
  "success": true,
  "data": { /* Created project */ }
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:PORT/api/projects \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Project",
    "git_repo_path": "/Users/me/repos/my-project",
    "use_existing_repo": true,
    "setup_script": "npm install",
    "dev_script": null,
    "cleanup_script": null,
    "copy_files": null
  }'
```

### Update Project

**Endpoint:** `PUT /api/projects/{id}`

**Description:** Update an existing project.

**Request Body:**
```json
{
  "name": "Updated Project Name",
  "git_repo_path": null,
  "setup_script": "pnpm install",
  "dev_script": "pnpm dev",
  "cleanup_script": null,
  "copy_files": null
}
```

**Note:** All fields are optional. Null values will keep existing values.

### Delete Project

**Endpoint:** `DELETE /api/projects/{id}`

**Description:** Delete a project (cascade deletes tasks and attempts).

**Response:**
```json
{
  "success": true,
  "data": null
}
```

### Get Project Branches

**Endpoint:** `GET /api/projects/{id}/branches`

**Description:** Get all git branches in the project repository.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "name": "main",
      "is_current": true,
      "is_remote": false,
      "last_commit_date": "2025-01-08T00:00:00Z"
    },
    {
      "name": "origin/develop",
      "is_current": false,
      "is_remote": true,
      "last_commit_date": "2025-01-07T00:00:00Z"
    }
  ]
}
```

### Search Project Files

**Endpoint:** `GET /api/projects/{id}/search?q=filename&mode=TaskForm`

**Description:** Search for files in project repository (with intelligent ranking).

**Query Parameters:**
- `q` (required): Search query string
- `mode` (optional): "TaskForm" (respects .gitignore) or "Settings" (includes ignored files)

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "path": "src/index.ts",
      "is_file": true,
      "match_type": "FileName"
    },
    {
      "path": "src/components/Header.tsx",
      "is_file": true,
      "match_type": "FullPath"
    }
  ]
}
```

**Match Types:**
- `FileName`: Filename matches query
- `DirectoryName`: Parent directory name matches
- `FullPath`: Full path contains query

**Note:** Results are ranked by git commit history (most frequently changed files first).

### Open Project in Editor

**Endpoint:** `POST /api/projects/{id}/open-editor`

**Description:** Open project directory in configured code editor.

**Request Body:**
```json
{
  "editor_type": null  // Optional override, e.g., "CURSOR"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "url": null  // URL for remote editors (SSH), null for local
  }
}
```

---

## Tasks

### List Tasks

**Endpoint:** `GET /api/tasks?project_id={uuid}`

**Description:** Get all tasks for a project.

**Query Parameters:**
- `project_id` (required): Project UUID

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "task-uuid",
      "project_id": "project-uuid",
      "title": "Fix login bug",
      "description": "Users cannot log in with special characters in password",
      "status": "inprogress",
      "parent_task_attempt": null,
      "created_at": "2025-01-08T00:00:00Z",
      "updated_at": "2025-01-08T00:00:00Z",
      "has_in_progress_attempt": true,
      "has_merged_attempt": false,
      "last_attempt_failed": false,
      "executor": "CLAUDE_CODE"
    }
  ]
}
```

**Task Status Values:**
- `todo` - Not started
- `inprogress` - Work in progress
- `inreview` - Ready for review
- `done` - Completed and merged
- `cancelled` - Cancelled

### Get Task

**Endpoint:** `GET /api/tasks/{task_id}`

**Description:** Get a specific task by ID.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "project_id": "project-uuid",
    "title": "Task title",
    "description": "Task description",
    "status": "todo",
    "parent_task_attempt": null,
    "created_at": "2025-01-08T00:00:00Z",
    "updated_at": "2025-01-08T00:00:00Z"
  }
}
```

### Create Task

**Endpoint:** `POST /api/tasks`

**Description:** Create a new task.

**Request Body:**
```json
{
  "project_id": "project-uuid",
  "title": "Implement user authentication",
  "description": "Add JWT-based authentication flow with refresh tokens",
  "parent_task_attempt": null,
  "image_ids": ["image-uuid-1", "image-uuid-2"]
}
```

**Field Details:**
- `project_id` (required): Parent project UUID
- `title` (required): Task title/summary
- `description` (optional): Detailed task description (supports markdown)
- `parent_task_attempt` (optional): UUID of parent task attempt (for subtasks)
- `image_ids` (optional): Array of image UUIDs to attach

**Response:**
```json
{
  "success": true,
  "data": { /* Created task */ }
}
```

### Create and Start Task

**Endpoint:** `POST /api/tasks/create-and-start`

**Description:** Create a task and immediately start a task attempt with an AI executor.

**Request Body:**
```json
{
  "task": {
    "project_id": "project-uuid",
    "title": "Add dark mode",
    "description": "Implement dark mode theme switcher",
    "parent_task_attempt": null,
    "image_ids": null
  },
  "executor_profile_id": {
    "executor": "CLAUDE_CODE",
    "variant": null
  },
  "base_branch": "main"
}
```

**Field Details:**
- `task`: CreateTask object (same as Create Task endpoint)
- `executor_profile_id`: Executor configuration
  - `executor`: BaseCodingAgent enum (CLAUDE_CODE, AMP, GEMINI, etc.)
  - `variant`: Optional variant name (e.g., "PLAN")
- `base_branch`: Git branch to base the work on

**Response:**
```json
{
  "success": true,
  "data": {
    /* TaskWithAttemptStatus object */
    "id": "task-uuid",
    "has_in_progress_attempt": true,
    "executor": "CLAUDE_CODE"
  }
}
```

### Update Task

**Endpoint:** `PUT /api/tasks/{task_id}`

**Description:** Update an existing task.

**Request Body:**
```json
{
  "title": "Updated task title",
  "description": "Updated description",
  "status": "inreview",
  "parent_task_attempt": null,
  "image_ids": ["new-image-uuid"]
}
```

**Note:** All fields are optional. Omitted fields keep existing values.

### Delete Task

**Endpoint:** `DELETE /api/tasks/{task_id}`

**Description:** Delete a task (cascade deletes attempts and processes).

**Response:**
```json
{
  "success": true,
  "data": null
}
```

**Status Code:** 202 Accepted (deletion happens in background)

**Note:** Worktree cleanup runs asynchronously to avoid blocking the response.

### Stream Tasks (WebSocket)

**Endpoint:** `WS /api/tasks/stream/ws?project_id={uuid}`

**Description:** Real-time task updates for a project via WebSocket.

**WebSocket Messages:**
```json
{
  "type": "task_created" | "task_updated" | "task_deleted",
  "data": { /* Task object */ }
}
```

---

## Task Attempts

Task attempts represent individual execution sessions for a task. Each attempt:
- Creates an isolated git worktree
- Runs in a separate branch
- Can invoke AI coding agents or scripts
- Tracks all execution processes

### List Task Attempts

**Endpoint:** `GET /api/task-attempts?task_id={uuid}`

**Description:** Get all attempts for a task.

**Query Parameters:**
- `task_id` (optional): Filter by task UUID

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "attempt-uuid",
      "task_id": "task-uuid",
      "container_ref": "/path/to/worktree",
      "branch": "anyon/task-abc123",
      "target_branch": "main",
      "executor": "CLAUDE_CODE",
      "worktree_deleted": false,
      "setup_completed_at": "2025-01-08T00:00:00Z",
      "created_at": "2025-01-08T00:00:00Z",
      "updated_at": "2025-01-08T00:00:00Z"
    }
  ]
}
```

### Get Task Attempt

**Endpoint:** `GET /api/task-attempts/{id}`

**Description:** Get a specific task attempt.

**Response:**
```json
{
  "success": true,
  "data": { /* TaskAttempt object */ }
}
```

### Create Task Attempt

**Endpoint:** `POST /api/task-attempts`

**Description:** Create a new task attempt and start initial execution.

**Request Body:**
```json
{
  "task_id": "task-uuid",
  "executor_profile_id": {
    "executor": "CLAUDE_CODE",
    "variant": "PLAN"
  },
  "base_branch": "main"
}
```

**Response:**
```json
{
  "success": true,
  "data": { /* Created TaskAttempt */ }
}
```

### Send Follow-up

**Endpoint:** `POST /api/task-attempts/{id}/follow-up`

**Description:** Send a follow-up message/instruction to the AI agent.

**Request Body:**
```json
{
  "prompt": "Please add error handling for edge cases",
  "variant": null,
  "image_ids": ["image-uuid"],
  "retry_process_id": null,
  "force_when_dirty": false,
  "perform_git_reset": true
}
```

**Field Details:**
- `prompt` (required): Follow-up instruction text
- `variant` (optional): Override executor variant
- `image_ids` (optional): Attach images to the message
- `retry_process_id` (optional): If set, this becomes a retry operation
- `force_when_dirty` (optional): Allow retry even with uncommitted changes
- `perform_git_reset` (optional): Whether to perform git reset during retry

**Response:**
```json
{
  "success": true,
  "data": { /* New ExecutionProcess */ }
}
```

### Replace Process (Retry from Point)

**Endpoint:** `POST /api/task-attempts/{id}/replace-process`

**Description:** Delete a process and all subsequent ones, then start a new execution from that point.

**Request Body:**
```json
{
  "process_id": "process-uuid",
  "prompt": "Try a different approach",
  "variant": null,
  "force_when_dirty": false,
  "perform_git_reset": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "deleted_count": 3,
    "git_reset_needed": true,
    "git_reset_applied": true,
    "target_before_oid": "abc123...",
    "new_execution_id": "new-process-uuid"
  }
}
```

### Run Agent Setup Helper

**Endpoint:** `POST /api/task-attempts/{id}/run-agent-setup`

**Description:** Run setup helper for agents that support it (e.g., Cursor agent setup).

**Request Body:**
```json
{
  "executor_profile_id": {
    "executor": "CURSOR_AGENT",
    "variant": null
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {}
}
```

### Get Branch Status

**Endpoint:** `GET /api/task-attempts/{id}/branch-status`

**Description:** Get detailed git branch status for the task attempt.

**Response:**
```json
{
  "success": true,
  "data": {
    "commits_ahead": 5,
    "commits_behind": 2,
    "has_uncommitted_changes": true,
    "head_oid": "abc123def456...",
    "uncommitted_count": 3,
    "untracked_count": 1,
    "target_branch_name": "main",
    "remote_commits_ahead": 0,
    "remote_commits_behind": 0,
    "merges": [],
    "is_rebase_in_progress": false,
    "conflict_op": null,
    "conflicted_files": []
  }
}
```

**Field Details:**
- `commits_ahead`: Commits ahead of target branch
- `commits_behind`: Commits behind target branch
- `has_uncommitted_changes`: Dirty working tree
- `head_oid`: Current HEAD commit SHA
- `uncommitted_count`: Number of modified files
- `untracked_count`: Number of untracked files
- `remote_commits_ahead/behind`: Status vs remote (for PR branches)
- `merges`: PR or direct merge records
- `is_rebase_in_progress`: Git rebase in progress
- `conflict_op`: Type of operation causing conflicts (rebase/merge/cherry_pick/revert)
- `conflicted_files`: List of files with merge conflicts

### Stream Diff (WebSocket)

**Endpoint:** `WS /api/task-attempts/{id}/diff/ws?stats_only=false`

**Description:** Real-time git diff streaming for the task attempt.

**Query Parameters:**
- `stats_only` (optional): If true, only send stats (additions/deletions), not full content

**WebSocket Messages:**
```json
{
  "type": "diff",
  "data": {
    "change": "added" | "deleted" | "modified" | "renamed",
    "oldPath": "src/old.ts",
    "newPath": "src/new.ts",
    "oldContent": "...",
    "newContent": "...",
    "contentOmitted": false,
    "additions": 10,
    "deletions": 5
  }
}
```

### Get Commit Info

**Endpoint:** `GET /api/task-attempts/{id}/commit-info?sha=abc123`

**Description:** Get commit subject/message for a specific SHA.

**Query Parameters:**
- `sha` (required): Git commit SHA

**Response:**
```json
{
  "success": true,
  "data": {
    "sha": "abc123def...",
    "subject": "feat: add user authentication"
  }
}
```

### Compare Commit to HEAD

**Endpoint:** `GET /api/task-attempts/{id}/commit-compare?sha=abc123`

**Description:** Compare a commit to current HEAD to determine if it's in history.

**Response:**
```json
{
  "success": true,
  "data": {
    "head_oid": "def456...",
    "target_oid": "abc123...",
    "ahead_from_head": 0,
    "behind_from_head": 3,
    "is_linear": true
  }
}
```

### Merge to Target Branch

**Endpoint:** `POST /api/task-attempts/{id}/merge`

**Description:** Merge task attempt changes directly to target branch (local merge).

**Response:**
```json
{
  "success": true,
  "data": null
}
```

**Note:** Creates a merge commit in the main repo and marks task as done.

### Push Branch to GitHub

**Endpoint:** `POST /api/task-attempts/{id}/push`

**Description:** Push the task attempt branch to GitHub remote.

**Response:**
```json
{
  "success": true,
  "data": null
}
```

**Note:** Requires valid GitHub token in config.

### Create GitHub Pull Request

**Endpoint:** `POST /api/task-attempts/{id}/pr`

**Description:** Push branch and create a GitHub pull request.

**Request Body:**
```json
{
  "title": "Add dark mode support",
  "body": "This PR implements dark mode theme switching.\n\nCloses #123",
  "target_branch": null  // Optional, defaults to attempt's target_branch
}
```

**Response:**
```json
{
  "success": true,
  "data": "https://github.com/user/repo/pull/456"
}
```

**Note:** Auto-opens PR in browser after creation.

### Attach Existing Pull Request

**Endpoint:** `POST /api/task-attempts/{id}/pr/attach`

**Description:** Find and attach an existing PR for this branch.

**Response:**
```json
{
  "success": true,
  "data": {
    "pr_attached": true,
    "pr_url": "https://github.com/user/repo/pull/456",
    "pr_number": 456,
    "pr_status": "open"
  }
}
```

### Rebase on Target Branch

**Endpoint:** `POST /api/task-attempts/{id}/rebase`

**Description:** Rebase the task attempt branch onto target branch.

**Request Body:**
```json
{
  "old_base_branch": null,  // Optional, defaults to current target_branch
  "new_base_branch": null   // Optional, defaults to current target_branch
}
```

**Response (Success):**
```json
{
  "success": true,
  "data": null
}
```

**Response (Conflicts):**
```json
{
  "success": false,
  "error_data": {
    "type": "merge_conflicts",
    "message": "Conflicts in src/index.ts",
    "op": "rebase"
  }
}
```

### Abort Conflicts

**Endpoint:** `POST /api/task-attempts/{id}/conflicts/abort`

**Description:** Abort ongoing rebase/merge and return to clean state.

**Response:**
```json
{
  "success": true,
  "data": null
}
```

### Change Target Branch

**Endpoint:** `POST /api/task-attempts/{id}/change-target-branch`

**Description:** Update the target branch for this attempt.

**Request Body:**
```json
{
  "new_target_branch": "develop"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "new_target_branch": "develop",
    "status": [5, 2]  // [ahead, behind]
  }
}
```

### Rename Branch

**Endpoint:** `POST /api/task-attempts/{id}/rename-branch`

**Description:** Rename the git branch for this attempt.

**Request Body:**
```json
{
  "new_branch_name": "feature/dark-mode"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "branch": "feature/dark-mode"
  }
}
```

**Note:** Cannot rename if PR is open. Updates child task attempts if any.

### Open in Editor

**Endpoint:** `POST /api/task-attempts/{id}/open-editor`

**Description:** Open task attempt worktree in code editor.

**Request Body:**
```json
{
  "editor_type": null,  // Optional override
  "file_path": null     // Optional specific file to open
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "url": null  // URL for remote editors
  }
}
```

### Delete File

**Endpoint:** `POST /api/task-attempts/{id}/delete-file?file_path=src/unused.ts`

**Description:** Delete a file from the worktree and commit the deletion.

**Query Parameters:**
- `file_path` (required): Relative path to file

**Response:**
```json
{
  "success": true,
  "data": null
}
```

### Start Dev Server

**Endpoint:** `POST /api/task-attempts/{id}/start-dev-server`

**Description:** Start the project's dev server script in this attempt's worktree.

**Response:**
```json
{
  "success": true,
  "data": null
}
```

**Note:** Stops any existing dev servers for the project first.

### Get Children Tasks

**Endpoint:** `GET /api/task-attempts/{id}/children`

**Description:** Get task relationships (parent task and child tasks).

**Response:**
```json
{
  "success": true,
  "data": {
    "parent_task": { /* Task object */ },
    "current_attempt": { /* TaskAttempt object */ },
    "children": [ /* Array of child Task objects */ ]
  }
}
```

### Stop Execution

**Endpoint:** `POST /api/task-attempts/{id}/stop`

**Description:** Stop all running execution processes for this attempt.

**Response:**
```json
{
  "success": true,
  "data": null
}
```

### Draft Management

#### Get Draft

**Endpoint:** `GET /api/task-attempts/{id}/draft`

**Description:** Get saved draft message for follow-up or retry.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "draft-uuid",
    "task_attempt_id": "attempt-uuid",
    "draft_type": "follow_up" | "retry",
    "retry_process_id": "process-uuid",
    "prompt": "Draft message text",
    "queued": false,
    "sending": false,
    "variant": null,
    "image_ids": [],
    "version": 1,
    "created_at": "2025-01-08T00:00:00Z",
    "updated_at": "2025-01-08T00:00:00Z"
  }
}
```

#### Save Draft

**Endpoint:** `PUT /api/task-attempts/{id}/draft`

**Description:** Save or update a draft message.

**Request Body:**
```json
{
  "prompt": "Try implementing this with...",
  "variant": null,
  "image_ids": ["image-uuid"],
  "version": 1
}
```

**Response:**
```json
{
  "success": true,
  "data": { /* Updated Draft */ }
}
```

#### Delete Draft

**Endpoint:** `DELETE /api/task-attempts/{id}/draft`

**Description:** Delete the saved draft.

**Response:**
```json
{
  "success": true,
  "data": null
}
```

#### Queue Draft for Auto-Send

**Endpoint:** `POST /api/task-attempts/{id}/draft/queue`

**Description:** Mark draft as queued for automatic sending.

**Response:**
```json
{
  "success": true,
  "data": { /* Updated Draft with queued=true */ }
}
```

---

## Execution Processes

Execution processes represent individual runs of AI agents or scripts.

### List Execution Processes

**Endpoint:** `GET /api/execution-processes?task_attempt_id={uuid}&show_soft_deleted=false`

**Description:** Get all execution processes for a task attempt.

**Query Parameters:**
- `task_attempt_id` (required): Task attempt UUID
- `show_soft_deleted` (optional): Include dropped/trimmed processes (default: false)

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "process-uuid",
      "task_attempt_id": "attempt-uuid",
      "run_reason": "codingagent",
      "executor_action": {
        "typ": { "type": "CodingAgentInitialRequest", /* ... */ },
        "next_action": null
      },
      "before_head_commit": "abc123...",
      "after_head_commit": "def456...",
      "status": "completed",
      "exit_code": 0,
      "dropped": false,
      "started_at": "2025-01-08T00:00:00Z",
      "completed_at": "2025-01-08T00:05:30Z",
      "created_at": "2025-01-08T00:00:00Z",
      "updated_at": "2025-01-08T00:05:30Z"
    }
  ]
}
```

**Run Reason Values:**
- `setupscript` - Setup script execution
- `cleanupscript` - Cleanup script execution
- `codingagent` - AI coding agent execution
- `devserver` - Dev server process

**Status Values:**
- `running` - Currently executing
- `completed` - Finished successfully
- `failed` - Exited with error
- `killed` - Manually stopped

### Get Execution Process

**Endpoint:** `GET /api/execution-processes/{id}`

**Description:** Get a specific execution process.

**Response:**
```json
{
  "success": true,
  "data": { /* ExecutionProcess object */ }
}
```

### Stop Execution Process

**Endpoint:** `POST /api/execution-processes/{id}/stop`

**Description:** Stop a running execution process.

**Response:**
```json
{
  "success": true,
  "data": null
}
```

### Stream Raw Logs (WebSocket)

**Endpoint:** `WS /api/execution-processes/{id}/raw-logs/ws`

**Description:** Stream raw stdout/stderr logs via WebSocket.

**WebSocket Messages (JSON Patches):**
```json
{
  "op": "add",
  "path": "/0",
  "value": {
    "type": "stdout",
    "content": "Installing dependencies..."
  }
}
```

**Note:** Messages are JSON Patches for efficient updates.

### Stream Normalized Logs (WebSocket)

**Endpoint:** `WS /api/execution-processes/{id}/normalized-logs/ws`

**Description:** Stream parsed, structured logs (AI messages, tool uses, etc.).

**WebSocket Messages:**
```json
{
  "timestamp": "2025-01-08T00:00:00Z",
  "entry_type": {
    "type": "tool_use",
    "tool_name": "Write",
    "action_type": {
      "action": "file_edit",
      "path": "src/index.ts",
      "changes": [{ /* ... */ }]
    },
    "status": { "status": "success" }
  },
  "content": "Writing to src/index.ts"
}
```

**Entry Types:**
- `user_message` - User input
- `assistant_message` - AI response
- `tool_use` - Tool execution (file edit, command, etc.)
- `error_message` - Errors
- `thinking` - AI reasoning process
- `next_action` - Action planning
- `system_message` - System info

### Stream Execution Processes (WebSocket)

**Endpoint:** `WS /api/execution-processes/stream/ws?task_attempt_id={uuid}&show_soft_deleted=false`

**Description:** Real-time updates for execution processes of a task attempt.

**WebSocket Messages:**
```json
{
  "type": "process_created" | "process_updated" | "process_completed",
  "data": { /* ExecutionProcess object */ }
}
```

---

## Images

### Upload Image

**Endpoint:** `POST /api/images/upload`

**Description:** Upload an image file.

**Request:** Multipart form data with `image` field

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "image-uuid",
    "file_path": ".vibe/images/abc123.png",
    "original_name": "screenshot.png",
    "mime_type": "image/png",
    "size_bytes": 123456,
    "hash": "sha256:abc123...",
    "created_at": "2025-01-08T00:00:00Z",
    "updated_at": "2025-01-08T00:00:00Z"
  }
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:PORT/api/images/upload \
  -F "image=@screenshot.png"
```

### Upload Task Image

**Endpoint:** `POST /api/images/task/{task_id}/upload`

**Description:** Upload and automatically associate image with a task.

**Request:** Multipart form data with `image` field

**Response:**
```json
{
  "success": true,
  "data": { /* ImageResponse */ }
}
```

### Get Image File

**Endpoint:** `GET /api/images/{id}/file`

**Description:** Serve the actual image file.

**Response:** Image file (PNG, JPEG, etc.)

**Example:**
```bash
curl http://127.0.0.1:PORT/api/images/{uuid}/file -o image.png
```

### Delete Image

**Endpoint:** `DELETE /api/images/{id}`

**Description:** Delete an image and its file.

**Response:**
```json
{
  "success": true,
  "data": null
}
```

### Get Task Images

**Endpoint:** `GET /api/images/task/{task_id}`

**Description:** Get all images associated with a task.

**Response:**
```json
{
  "success": true,
  "data": [
    { /* ImageResponse */ }
  ]
}
```

---

## Events (SSE/WebSocket)

### Global Event Stream

**Endpoint:** `GET /api/events` (Server-Sent Events)

**Description:** Subscribe to all system events via SSE.

**Event Types:**
- Task created/updated/deleted
- Task attempt created/updated
- Execution process started/completed
- PR status changed

**Example:**
```bash
curl -N http://127.0.0.1:PORT/api/events
```

**SSE Format:**
```
event: task_created
data: {"id":"uuid", "title":"...",...}

event: process_completed
data: {"id":"uuid", "status":"completed",...}
```

---

## Filesystem

### List Directory

**Endpoint:** `GET /api/filesystem/directory?path=/Users/me/projects`

**Description:** List contents of a directory.

**Query Parameters:**
- `path` (optional): Directory path (defaults to home directory)

**Response:**
```json
{
  "success": true,
  "data": {
    "entries": [
      {
        "name": "my-project",
        "path": "/Users/me/projects/my-project",
        "is_directory": true,
        "is_git_repo": true,
        "last_modified": 1704672000000
      }
    ],
    "current_path": "/Users/me/projects"
  }
}
```

### List Git Repositories

**Endpoint:** `GET /api/filesystem/git-repos?path=/Users/me`

**Description:** Recursively find git repositories in a directory.

**Query Parameters:**
- `path` (optional): Starting directory (defaults to common locations)

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "name": "anyon",
      "path": "/Users/me/projects/anyon",
      "is_directory": true,
      "is_git_repo": true,
      "last_modified": 1704672000000
    }
  ]
}
```

**Note:** Limited to prevent performance issues (max depth, max entries).

---

## Containers

### Get Container Info

**Endpoint:** `GET /api/containers/info?ref=/path/to/worktree`

**Description:** Resolve container (worktree) path to task/project info.

**Query Parameters:**
- `ref` (required): Container reference (worktree path)

**Response:**
```json
{
  "success": true,
  "data": {
    "attempt_id": "attempt-uuid",
    "task_id": "task-uuid",
    "project_id": "project-uuid"
  }
}
```

---

## Tags

Tags are reusable text snippets (prompts, instructions, etc.).

### List Tags

**Endpoint:** `GET /api/tags?search=query`

**Description:** Get all tags, optionally filtered by search query.

**Query Parameters:**
- `search` (optional): Search query

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "tag-uuid",
      "tag_name": "code-review",
      "content": "Please review this code for...",
      "created_at": "2025-01-08T00:00:00Z",
      "updated_at": "2025-01-08T00:00:00Z"
    }
  ]
}
```

### Get Tag

**Endpoint:** `GET /api/tags/{id}`

**Description:** Get a specific tag.

**Response:**
```json
{
  "success": true,
  "data": { /* Tag object */ }
}
```

### Create Tag

**Endpoint:** `POST /api/tags`

**Description:** Create a new tag.

**Request Body:**
```json
{
  "tag_name": "testing-prompt",
  "content": "Please write comprehensive unit tests for..."
}
```

**Response:**
```json
{
  "success": true,
  "data": { /* Created tag */ }
}
```

### Update Tag

**Endpoint:** `PUT /api/tags/{id}`

**Description:** Update an existing tag.

**Request Body:**
```json
{
  "tag_name": "updated-name",
  "content": "Updated content"
}
```

### Delete Tag

**Endpoint:** `DELETE /api/tags/{id}`

**Description:** Delete a tag.

**Response:**
```json
{
  "success": true,
  "data": null
}
```

---

## Approvals

Approvals are required when AI agents need permission to execute certain tools.

### Respond to Approval

**Endpoint:** `POST /api/approvals/{id}/respond`

**Description:** Approve or deny a tool execution request.

**Request Body:**
```json
{
  "status": {
    "status": "approved"
  }
}
```

**OR:**

```json
{
  "status": {
    "status": "denied",
    "reason": "Too risky"
  }
}
```

**Response:**
```json
{
  "status": "approved" | "denied" | "pending" | "timed_out"
}
```

**Approval Status:**
- `pending` - Awaiting user response
- `approved` - User approved
- `denied` - User rejected
- `timed_out` - Request expired

---

## Common Patterns

### Pagination

Currently, pagination is not implemented. All list endpoints return complete results.

### Filtering

Where applicable, query parameters provide filtering (e.g., `task_id`, `project_id`).

### Sorting

Results are returned in database order (typically by `created_at DESC`).

### Batch Operations

Not currently supported. Use individual requests.

---

## Response Format

All endpoints follow a consistent response format:

**Success:**
```json
{
  "success": true,
  "data": { /* Response data */ },
  "error_data": null,
  "message": null
}
```

**Error:**
```json
{
  "success": false,
  "data": null,
  "error_data": { /* Optional structured error data */ },
  "message": "Error description"
}
```

---

## TypeScript Types

All request/response types are auto-generated from Rust structs in `/shared/types.ts`. Import them in your frontend:

```typescript
import type {
  Task,
  CreateTask,
  TaskAttempt,
  ExecutionProcess
} from '../shared/types';
```
