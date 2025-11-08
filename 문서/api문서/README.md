# Anyon API Documentation

Welcome to the Anyon API documentation. Anyon is a task management and AI-powered code execution platform that integrates with git repositories and AI coding agents.

## Table of Contents

- [Overview](#overview)
- [Base URL](#base-url)
- [Authentication](#authentication)
- [API Endpoints](#api-endpoints)
- [WebSocket Endpoints](#websocket-endpoints)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)

## Overview

The Anyon API provides programmatic access to:
- Project management
- Task creation and tracking
- Task attempts with git worktree isolation
- Execution processes (AI agent runs, scripts)
- Real-time event streaming via SSE and WebSocket
- GitHub integration (OAuth, PR creation)
- File and image management
- MCP (Model Context Protocol) server integration

## Base URL

```
http://127.0.0.1:{PORT}/api
```

The backend port is auto-assigned by default. Check the server startup logs or read the port file at `~/.anyon/port` to find the actual port.

**Environment Variables:**
- `BACKEND_PORT` - Set specific backend port (default: auto-assign)
- `HOST` - Backend host (default: 127.0.0.1)

## Authentication

### GitHub OAuth (Device Flow)

Anyon uses GitHub OAuth for user authentication. The device flow is used for CLI/desktop applications.

**Flow:**
1. Start device flow: `POST /api/auth/github/device/start`
2. User authorizes via browser
3. Poll for token: `POST /api/auth/github/device/poll`
4. Token stored in config file

See [authentication.md](./authentication.md) for detailed flow.

## API Endpoints

All endpoints are documented in detail in [endpoints.md](./endpoints.md).

### Quick Reference

| Category | Endpoints |
|----------|-----------|
| **Projects** | `/api/projects/*` |
| **Tasks** | `/api/tasks/*` |
| **Task Attempts** | `/api/task-attempts/*` |
| **Execution Processes** | `/api/execution-processes/*` |
| **Images** | `/api/images/*` |
| **Config** | `/api/info`, `/api/config`, `/api/mcp-config` |
| **Auth** | `/api/auth/github/*` |
| **Events** | `/api/events/*` (SSE/WebSocket) |
| **Filesystem** | `/api/filesystem/*` |

## WebSocket Endpoints

Anyon provides real-time updates via WebSocket connections:

- `/api/tasks/stream/ws` - Task updates for a project
- `/api/task-attempts/{id}/diff/ws` - Real-time git diff streaming
- `/api/execution-processes/stream/ws` - Execution process updates
- `/api/execution-processes/{id}/raw-logs/ws` - Raw stdout/stderr logs
- `/api/execution-processes/{id}/normalized-logs/ws` - Parsed, structured logs

## Error Handling

All API responses follow a consistent format:

```json
{
  "success": true,
  "data": { ... },
  "error_data": null,
  "message": null
}
```

For errors:

```json
{
  "success": false,
  "data": null,
  "error_data": { ... },
  "message": "Error description"
}
```

See [errors.md](./errors.md) for complete error code reference.

## Rate Limiting

Currently, no rate limiting is enforced on the local API. Future cloud deployments may implement rate limits.

## Getting Started

### Example: Create a Project and Task

```bash
# 1. Create a project
curl -X POST http://127.0.0.1:PORT/api/projects \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Project",
    "git_repo_path": "/path/to/repo",
    "use_existing_repo": true,
    "setup_script": null,
    "dev_script": null,
    "cleanup_script": null,
    "copy_files": null
  }'

# 2. Create a task
curl -X POST http://127.0.0.1:PORT/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "project_id": "PROJECT_UUID",
    "title": "Fix login bug",
    "description": "Users cannot log in with special characters",
    "parent_task_attempt": null,
    "image_ids": null
  }'

# 3. Start a task attempt with AI agent
curl -X POST http://127.0.0.1:PORT/api/tasks/create-and-start \
  -H "Content-Type: application/json" \
  -d '{
    "task": {
      "project_id": "PROJECT_UUID",
      "title": "Implement feature X",
      "description": "Add new authentication flow"
    },
    "executor_profile_id": {
      "executor": "CLAUDE_CODE",
      "variant": null
    },
    "base_branch": "main"
  }'
```

## API Design Principles

1. **Type-Safe**: TypeScript types are auto-generated from Rust structs via `ts-rs`
2. **RESTful**: Resource-based URLs with standard HTTP methods
3. **Real-time**: WebSocket and SSE for live updates
4. **Git-Centric**: Each task attempt runs in isolated git worktree
5. **Streaming**: Large responses (logs, diffs) use streaming endpoints

## Additional Resources

- [OpenAPI Specification](./openapi.yaml) - Complete API spec
- [Endpoint Reference](./endpoints.md) - All endpoints with examples
- [Authentication Guide](./authentication.md) - Auth flow details
- [Error Reference](./errors.md) - Error codes and handling
- [Examples](./examples/) - Code examples for common operations

## Support

For issues or questions:
- GitHub Issues: https://github.com/your-org/anyon/issues
- Documentation: See project README.md
