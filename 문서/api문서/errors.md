# Error Reference

Complete guide to error handling in the Anyon API.

## Table of Contents

- [Error Response Format](#error-response-format)
- [HTTP Status Codes](#http-status-codes)
- [Error Categories](#error-categories)
- [GitHub Service Errors](#github-service-errors)
- [Git Operation Errors](#git-operation-errors)
- [Validation Errors](#validation-errors)
- [Database Errors](#database-errors)
- [File System Errors](#filesystem-errors)
- [Executor Errors](#executor-errors)
- [Error Handling Best Practices](#error-handling-best-practices)

---

## Error Response Format

All API errors follow a consistent format:

```typescript
{
  success: false,
  data: null,
  error_data: ErrorType | null,  // Optional structured error data
  message: string | null           // Human-readable error message
}
```

### Basic Error Example

```json
{
  "success": false,
  "data": null,
  "error_data": null,
  "message": "Task not found"
}
```

### Structured Error Example

```json
{
  "success": false,
  "data": null,
  "error_data": {
    "type": "merge_conflicts",
    "message": "Conflicts in src/index.ts, src/utils.ts",
    "op": "rebase"
  },
  "message": "Merge conflicts detected"
}
```

---

## HTTP Status Codes

Anyon uses standard HTTP status codes:

### Success Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful GET, PUT request |
| 201 | Created | Successful POST creating a resource |
| 202 | Accepted | Request accepted, processing asynchronously |
| 204 | No Content | Successful DELETE |

### Client Error Codes

| Code | Meaning | Common Causes |
|------|---------|---------------|
| 400 | Bad Request | Invalid request body, missing required fields |
| 404 | Not Found | Resource doesn't exist (project, task, etc.) |
| 409 | Conflict | Resource conflict (duplicate, running process, etc.) |
| 422 | Unprocessable Entity | Validation failed |

### Server Error Codes

| Code | Meaning | Common Causes |
|------|---------|---------------|
| 500 | Internal Server Error | Unexpected server error |
| 503 | Service Unavailable | Temporary service issue |

---

## Error Categories

### 1. Resource Not Found

**HTTP Status:** 404

**Response:**
```json
{
  "success": false,
  "data": null,
  "error_data": null,
  "message": "Project not found" | "Task not found" | "Task attempt not found"
}
```

**Common Endpoints:**
- `GET /api/projects/{id}`
- `GET /api/tasks/{id}`
- `GET /api/task-attempts/{id}`
- `GET /api/execution-processes/{id}`

**Recovery:**
- Verify the UUID is correct
- Check if the resource was deleted
- Refresh your local data

---

### 2. Validation Errors

**HTTP Status:** 400 or 200 with error message

**Examples:**

**Invalid Branch Prefix:**
```json
{
  "success": false,
  "data": null,
  "message": "Invalid git branch prefix. Must be a valid git branch name component without slashes."
}
```

**Empty Required Field:**
```json
{
  "success": false,
  "data": null,
  "message": "Branch name cannot be empty"
}
```

**Invalid Branch Name:**
```json
{
  "success": false,
  "data": null,
  "message": "Invalid branch name format"
}
```

**Duplicate Resource:**
```json
{
  "success": false,
  "data": null,
  "message": "A project with this git repository path already exists"
}
```

**Recovery:**
- Check field format and requirements
- Ensure unique constraints are met
- Validate input before sending

---

### 3. Conflict Errors

**HTTP Status:** 409

**Running Process Conflict:**
```json
{
  "success": false,
  "data": null,
  "message": "Task has running execution processes. Please wait for them to complete or stop them first."
}
```

**Endpoints:**
- `DELETE /api/tasks/{id}`

**Recovery:**
- Stop running processes first: `POST /api/task-attempts/{id}/stop`
- Wait for processes to complete
- Then retry the operation

**Open PR Conflict:**
```json
{
  "success": false,
  "data": null,
  "message": "Cannot rename branch with an open pull request. Please close the PR first or create a new attempt."
}
```

**Endpoints:**
- `POST /api/task-attempts/{id}/rename-branch`

**Recovery:**
- Close the PR on GitHub
- Or create a new task attempt
- Then retry the operation

---

## GitHub Service Errors

### Error Enum

```typescript
enum GitHubServiceError {
  TOKEN_INVALID = "TOKEN_INVALID",
  INSUFFICIENT_PERMISSIONS = "INSUFFICIENT_PERMISSIONS",
  REPO_NOT_FOUND_OR_NO_ACCESS = "REPO_NOT_FOUND_OR_NO_ACCESS"
}
```

### 1. Invalid Token

**Response:**
```json
{
  "success": false,
  "error_data": "TOKEN_INVALID",
  "message": "GitHub token is invalid or expired"
}
```

**Causes:**
- Token was revoked on GitHub
- Token expired (PAT only)
- No token configured

**Common Endpoints:**
- `POST /api/task-attempts/{id}/push`
- `POST /api/task-attempts/{id}/pr`
- `GET /api/auth/github/check`

**Recovery:**
1. Re-authenticate: `POST /api/auth/github/device/start`
2. Or configure new PAT in config
3. Verify: `GET /api/auth/github/check`

### 2. Insufficient Permissions

**Response:**
```json
{
  "success": false,
  "error_data": "INSUFFICIENT_PERMISSIONS",
  "message": "Token lacks required permissions (repo, user)"
}
```

**Causes:**
- Token missing `repo` scope
- Token missing `user` scope
- Token permissions changed

**Recovery:**
1. Revoke old token on GitHub
2. Re-authenticate with correct scopes
3. Or generate new PAT with `repo` + `user` scopes

### 3. Repository Not Found

**Response:**
```json
{
  "success": false,
  "error_data": "REPO_NOT_FOUND_OR_NO_ACCESS",
  "message": "Repository not found or token has no access"
}
```

**Causes:**
- Repository is private and token lacks access
- Repository was deleted
- Repository URL/path is incorrect
- Organization restricted access

**Common Endpoints:**
- `POST /api/task-attempts/{id}/push`
- `POST /api/task-attempts/{id}/pr`

**Recovery:**
1. Verify repository exists on GitHub
2. Check token has access to the repository
3. Verify repository URL in project settings
4. Check organization access policies

---

## Git Operation Errors

### Merge Conflicts

**Response:**
```json
{
  "success": false,
  "error_data": {
    "type": "merge_conflicts",
    "message": "Conflicts in src/index.ts, src/utils.ts",
    "op": "rebase"
  },
  "message": "Merge conflicts detected during rebase"
}
```

**Conflict Operations:**
```typescript
type ConflictOp = "rebase" | "merge" | "cherry_pick" | "revert";
```

**Endpoints:**
- `POST /api/task-attempts/{id}/rebase`

**Recovery:**
1. Check branch status: `GET /api/task-attempts/{id}/branch-status`
2. Review conflicted files in `conflicted_files` array
3. Manually resolve conflicts in the worktree
4. Or abort: `POST /api/task-attempts/{id}/conflicts/abort`

### Rebase In Progress

**Response:**
```json
{
  "success": false,
  "error_data": {
    "type": "rebase_in_progress"
  },
  "message": "A rebase is already in progress"
}
```

**Endpoints:**
- `POST /api/task-attempts/{id}/rebase`
- `POST /api/task-attempts/{id}/rename-branch`

**Recovery:**
1. Abort ongoing rebase: `POST /api/task-attempts/{id}/conflicts/abort`
2. Or complete the rebase manually in the worktree
3. Then retry the operation

### Branch Doesn't Exist

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "Branch 'feature-xyz' does not exist in the repository"
}
```

**Endpoints:**
- `POST /api/task-attempts/{id}/change-target-branch`
- `POST /api/task-attempts/{id}/rebase`

**Recovery:**
1. List available branches: `GET /api/projects/{id}/branches`
2. Use an existing branch name
3. Or create the branch in the main repository first

### Branch Name Invalid

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "A branch with this name already exists"
}
```

**Endpoints:**
- `POST /api/task-attempts/{id}/rename-branch`

**Recovery:**
- Choose a different branch name
- Or delete the existing branch first

---

## Validation Errors

### Missing Required Fields

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "Missing required field: project_id"
}
```

**Recovery:**
- Check API documentation for required fields
- Include all required fields in request body

### Invalid Field Format

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "Invalid executor profile format"
}
```

**Common Field Errors:**
- Invalid UUID format
- Invalid enum value (e.g., task status, executor type)
- Invalid JSON format

**Recovery:**
- Validate field format before sending
- Use TypeScript types from `/shared/types.ts`
- Check enum values match API specification

### Empty Search Query

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "Query parameter 'q' is required and cannot be empty"
}
```

**Endpoints:**
- `GET /api/projects/{id}/search`

**Recovery:**
- Provide non-empty search query
- Trim whitespace before sending

---

## Database Errors

### Row Not Found

**HTTP Status:** 404 or 500

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "Database error: row not found"
}
```

**Causes:**
- Resource was deleted
- Invalid UUID
- Database inconsistency

**Recovery:**
- Verify resource exists
- Refresh your data
- Check for concurrent deletions

### Foreign Key Constraint

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "Cannot delete: resource has dependent records"
}
```

**Causes:**
- Trying to delete project with tasks
- Database referential integrity

**Recovery:**
- Delete dependent records first
- Or use cascade delete endpoints

---

## Filesystem Errors

### Directory Not Found

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "Directory does not exist"
}
```

**Endpoints:**
- `GET /api/filesystem/directory`
- `GET /api/filesystem/git-repos`

**Recovery:**
- Verify directory path
- Use absolute path
- Check permissions

### Path Not a Directory

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "Path is not a directory"
}
```

**Endpoints:**
- `GET /api/filesystem/directory`
- `POST /api/projects` (when creating)

**Recovery:**
- Verify path points to a directory
- Not a file or symlink

### Not a Git Repository

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "The specified directory is not a git repository"
}
```

**Endpoints:**
- `POST /api/projects` (with `use_existing_repo: true`)

**Recovery:**
- Initialize git repo: `git init`
- Or set `use_existing_repo: false` to auto-initialize
- Verify `.git` directory exists

---

## Executor Errors

### Setup Helper Not Supported

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "This executor does not support setup helper"
}
```

**Endpoints:**
- `POST /api/task-attempts/{id}/run-agent-setup`

**Supported Executors:**
- `CURSOR_AGENT`

**Recovery:**
- Only use setup helper with supported executors
- Check capabilities: `GET /api/info`

### Executor Not Found

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "Executor not found"
}
```

**Endpoints:**
- `GET /api/mcp-config`
- Various executor-related endpoints

**Recovery:**
- Check executor profile ID is valid
- Verify executor is configured in profiles.json
- Use standard executor names (CLAUDE_CODE, AMP, etc.)

### MCP Not Supported

**Response:**
```json
{
  "success": false,
  "data": null,
  "message": "MCP not supported by this executor"
}
```

**Endpoints:**
- `GET /api/mcp-config`
- `POST /api/mcp-config`

**Supported Executors:**
- `CLAUDE_CODE`
- Others (check capabilities)

**Recovery:**
- Only configure MCP for supported executors
- Check `capabilities` field in system info

---

## Error Handling Best Practices

### 1. Always Check `success` Field

```typescript
const response = await fetch('/api/tasks', { method: 'POST', ... });
const json = await response.json();

if (!json.success) {
  // Handle error
  console.error(json.message);
  if (json.error_data) {
    // Handle structured error
    handleStructuredError(json.error_data);
  }
  return;
}

// Use json.data
const task = json.data;
```

### 2. Handle Structured Errors

```typescript
function handleGitOperationError(errorData: GitOperationError) {
  switch (errorData.type) {
    case 'merge_conflicts':
      // Show conflict resolution UI
      showConflictUI(errorData.message, errorData.op);
      break;
    case 'rebase_in_progress':
      // Prompt to abort or complete
      promptAbortRebase();
      break;
  }
}
```

### 3. Implement Retry Logic

```typescript
async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  maxRetries = 3,
  delay = 1000
): Promise<T> {
  let lastError;

  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;

      // Don't retry client errors (4xx)
      if (error.status >= 400 && error.status < 500) {
        throw error;
      }

      // Exponential backoff
      await new Promise(resolve => setTimeout(resolve, delay * Math.pow(2, i)));
    }
  }

  throw lastError;
}
```

### 4. User-Friendly Error Messages

```typescript
function getUserFriendlyMessage(error: ApiError): string {
  // GitHub errors
  if (error.error_data === 'TOKEN_INVALID') {
    return 'Your GitHub authentication has expired. Please sign in again.';
  }

  // Conflict errors
  if (error.message?.includes('running execution processes')) {
    return 'Cannot delete task while it is running. Please stop the task first.';
  }

  // Git errors
  if (error.error_data?.type === 'merge_conflicts') {
    return `Merge conflicts detected in: ${error.error_data.message}. Please resolve conflicts manually.`;
  }

  // Fallback
  return error.message || 'An unexpected error occurred. Please try again.';
}
```

### 5. Logging and Monitoring

```typescript
function logError(error: ApiError, context: string) {
  console.error(`[${context}] API Error:`, {
    message: error.message,
    errorData: error.error_data,
    timestamp: new Date().toISOString(),
  });

  // Send to error tracking service (e.g., Sentry)
  if (window.Sentry) {
    window.Sentry.captureException(error, {
      tags: { api_context: context },
      extra: { errorData: error.error_data },
    });
  }
}
```

---

## Common Error Scenarios

### Scenario 1: GitHub Push Fails

**Error:**
```json
{
  "success": false,
  "error_data": "TOKEN_INVALID",
  "message": "GitHub token is invalid"
}
```

**Solution:**
1. Check token: `GET /api/auth/github/check`
2. Re-authenticate: `POST /api/auth/github/device/start`
3. Retry push: `POST /api/task-attempts/{id}/push`

### Scenario 2: Cannot Delete Task

**Error:**
```json
{
  "success": false,
  "message": "Task has running execution processes..."
}
```

**Solution:**
1. Stop execution: `POST /api/task-attempts/{id}/stop`
2. Wait for processes to stop
3. Retry delete: `DELETE /api/tasks/{id}`

### Scenario 3: Rebase Conflicts

**Error:**
```json
{
  "error_data": {
    "type": "merge_conflicts",
    "op": "rebase"
  }
}
```

**Solution:**
1. Get status: `GET /api/task-attempts/{id}/branch-status`
2. Check `conflicted_files`
3. Open in editor: `POST /api/task-attempts/{id}/open-editor`
4. Resolve conflicts manually
5. Or abort: `POST /api/task-attempts/{id}/conflicts/abort`

### Scenario 4: Invalid Executor Profile

**Error:**
```json
{
  "success": false,
  "message": "Invalid executor profiles format"
}
```

**Solution:**
1. Get current profiles: `GET /api/profiles`
2. Validate JSON format
3. Check executor enum values
4. Update with valid data: `PUT /api/profiles`

---

## HTTP Client Configuration

### Recommended Timeout Settings

```typescript
const config = {
  timeout: 30000,        // 30 seconds for regular requests
  longTimeout: 300000,   // 5 minutes for long-running operations
};

// For streaming endpoints, no timeout:
const streamConfig = {
  timeout: 0,
};
```

### Operations That May Take Time

- Creating task attempts (sets up worktree)
- Rebasing large branches
- Pushing large repositories
- Creating PRs
- Searching files in large repositories

---

## Testing Error Handling

### Unit Test Example

```typescript
describe('Task API Error Handling', () => {
  it('should handle not found error', async () => {
    const response = await fetch('/api/tasks/invalid-uuid');
    const json = await response.json();

    expect(json.success).toBe(false);
    expect(json.message).toContain('not found');
  });

  it('should handle validation error', async () => {
    const response = await fetch('/api/tasks', {
      method: 'POST',
      body: JSON.stringify({ /* missing project_id */ }),
    });
    const json = await response.json();

    expect(json.success).toBe(false);
    expect(json.message).toContain('required');
  });
});
```

---

## Support

If you encounter an error not documented here:

1. Check server logs for detailed error information
2. Verify your request follows the API specification
3. Search existing GitHub issues
4. Create a new issue with:
   - Full error response
   - Request details (endpoint, method, body)
   - Steps to reproduce
   - Expected vs actual behavior
