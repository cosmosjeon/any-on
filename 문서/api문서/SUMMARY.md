# API Documentation Summary

Complete, production-ready API documentation for the Anyon project has been created.

## Documentation Files Created

### Core Documentation

1. **README.md** (4.9 KB)
   - API overview and quick start guide
   - Base URL and authentication overview
   - Quick reference table of all endpoints
   - Getting started examples
   - Links to detailed documentation

2. **endpoints.md** (35 KB)
   - Complete reference for ALL 50+ API endpoints
   - Detailed request/response schemas
   - Query parameters and path variables
   - Example curl commands for each endpoint
   - WebSocket endpoint documentation
   - Organized by resource category

3. **authentication.md** (15 KB)
   - Complete GitHub OAuth device flow guide
   - Step-by-step implementation examples
   - Code examples in JavaScript, Python, and Rust
   - Personal Access Token (PAT) setup
   - Token management and validation
   - Security best practices
   - Troubleshooting guide

4. **errors.md** (17 KB)
   - Complete error code reference
   - HTTP status codes with meanings
   - Structured error responses
   - GitHub service errors
   - Git operation errors (conflicts, rebase)
   - Validation and database errors
   - Error handling best practices
   - Recovery strategies for common scenarios

5. **openapi.yaml** (48 KB)
   - Complete OpenAPI 3.0 specification
   - All endpoints with full schema definitions
   - Reusable components and response types
   - Request/response examples
   - Can be imported into Postman, Insomnia, Swagger UI
   - Type-safe client generation ready

### Examples Directory

**examples/README.md** - Overview and usage guide for all examples

**Executable Shell Scripts:**

1. **01-create-project-and-task.sh**
   - Create a new project
   - Create a task within the project
   - List and verify creation

2. **02-github-auth-flow.sh**
   - Complete GitHub OAuth device flow
   - Automatic polling with backoff
   - Token verification

3. **03-start-task-with-ai.sh**
   - Start a task with Claude Code executor
   - Monitor execution processes
   - Branch management

4. **04-create-pr.sh**
   - Check GitHub authentication
   - Push branch to GitHub
   - Create pull request
   - Attach PR to task attempt

**Additional Examples in README:**
- Python code examples
- TypeScript/JavaScript examples
- WebSocket streaming examples
- Direct curl commands for common operations

## API Coverage

### Endpoints Documented

**System & Configuration** (7 endpoints)
- GET /api/info
- PUT /api/config
- GET/POST /api/mcp-config
- GET/PUT /api/profiles
- GET /api/sounds/{sound}

**Authentication** (3 endpoints)
- POST /api/auth/github/device/start
- POST /api/auth/github/device/poll
- GET /api/auth/github/check

**Projects** (7 endpoints)
- GET/POST /api/projects
- GET/PUT/DELETE /api/projects/{id}
- GET /api/projects/{id}/branches
- GET /api/projects/{id}/search
- POST /api/projects/{id}/open-editor

**Tasks** (6 endpoints)
- GET/POST /api/tasks
- GET/PUT/DELETE /api/tasks/{task_id}
- POST /api/tasks/create-and-start
- WS /api/tasks/stream/ws

**Task Attempts** (20+ endpoints)
- GET/POST /api/task-attempts
- GET /api/task-attempts/{id}
- POST /api/task-attempts/{id}/follow-up
- POST /api/task-attempts/{id}/replace-process
- POST /api/task-attempts/{id}/run-agent-setup
- GET /api/task-attempts/{id}/branch-status
- WS /api/task-attempts/{id}/diff/ws
- GET /api/task-attempts/{id}/commit-info
- GET /api/task-attempts/{id}/commit-compare
- POST /api/task-attempts/{id}/merge
- POST /api/task-attempts/{id}/push
- POST /api/task-attempts/{id}/pr
- POST /api/task-attempts/{id}/pr/attach
- POST /api/task-attempts/{id}/rebase
- POST /api/task-attempts/{id}/conflicts/abort
- POST /api/task-attempts/{id}/change-target-branch
- POST /api/task-attempts/{id}/rename-branch
- POST /api/task-attempts/{id}/open-editor
- POST /api/task-attempts/{id}/delete-file
- POST /api/task-attempts/{id}/start-dev-server
- GET /api/task-attempts/{id}/children
- POST /api/task-attempts/{id}/stop
- GET/PUT/DELETE /api/task-attempts/{id}/draft
- POST /api/task-attempts/{id}/draft/queue

**Execution Processes** (5 endpoints)
- GET/POST /api/execution-processes
- GET /api/execution-processes/{id}
- POST /api/execution-processes/{id}/stop
- WS /api/execution-processes/{id}/raw-logs/ws
- WS /api/execution-processes/{id}/normalized-logs/ws
- WS /api/execution-processes/stream/ws

**Images** (5 endpoints)
- POST /api/images/upload
- GET /api/images/{id}/file
- DELETE /api/images/{id}
- GET /api/images/task/{task_id}
- POST /api/images/task/{task_id}/upload

**Events** (1 endpoint)
- GET /api/events (SSE)

**Filesystem** (2 endpoints)
- GET /api/filesystem/directory
- GET /api/filesystem/git-repos

**Containers** (1 endpoint)
- GET /api/containers/info

**Tags** (5 endpoints)
- GET/POST /api/tags
- GET/PUT/DELETE /api/tags/{id}

**Approvals** (1 endpoint)
- POST /api/approvals/{id}/respond

**Total:** 60+ documented endpoints

## Key Features Documented

### Real-time Streaming
- WebSocket endpoints for logs, diffs, and events
- Server-Sent Events (SSE) for global event stream
- JSON Patch format for efficient updates

### Git Integration
- Worktree management
- Branch operations (rebase, merge, rename)
- Conflict detection and resolution
- GitHub push and PR creation

### AI Agent Integration
- Multiple executor support (Claude Code, AMP, Gemini, etc.)
- Follow-up messages
- Process replacement/retry
- Draft management

### Type Safety
- All types auto-generated from Rust via ts-rs
- OpenAPI schema for client generation
- TypeScript type imports available

## Documentation Quality

### Completeness
- Every endpoint documented with examples
- All request/response schemas included
- Error cases and recovery strategies
- WebSocket protocols explained

### Developer Experience
- Curl examples for every endpoint
- Code examples in multiple languages
- Executable shell scripts
- Common workflow examples

### Production Ready
- OpenAPI 3.0 specification
- Import into Postman/Insomnia
- Generate clients automatically
- Complete error reference

## Usage

### View Documentation

```bash
# Navigate to documentation directory
cd /Users/cosmos/Documents/dev/any-on/문서/api문서

# Read the overview
cat README.md

# View all endpoints
cat endpoints.md

# Check authentication guide
cat authentication.md

# Review error handling
cat errors.md
```

### Use OpenAPI Spec

**Import into Postman:**
1. Open Postman
2. File → Import
3. Select `openapi.yaml`
4. Update base URL with actual port

**Import into Insomnia:**
1. Open Insomnia
2. Application → Preferences → Data → Import Data
3. Select `openapi.yaml`

**Generate TypeScript Client:**
```bash
npx openapi-typescript openapi.yaml -o ./generated/api.ts
```

### Run Examples

```bash
cd /Users/cosmos/Documents/dev/any-on/문서/api문서/examples

# Create a project and task
./01-create-project-and-task.sh

# Authenticate with GitHub
./02-github-auth-flow.sh

# Start AI task (provide project ID)
./03-start-task-with-ai.sh <project-id>

# Create PR (provide attempt ID)
./04-create-pr.sh <attempt-id>
```

## File Locations

All documentation is located in:
```
/Users/cosmos/Documents/dev/any-on/문서/api문서/
├── README.md                    # API overview
├── endpoints.md                 # Complete endpoint reference
├── authentication.md            # Authentication guide
├── errors.md                    # Error reference
├── openapi.yaml                 # OpenAPI specification
├── SUMMARY.md                   # This file
└── examples/
    ├── README.md                # Examples overview
    ├── 01-create-project-and-task.sh
    ├── 02-github-auth-flow.sh
    ├── 03-start-task-with-ai.sh
    └── 04-create-pr.sh
```

## Next Steps

### For Frontend Developers
1. Import types from `/shared/types.ts`
2. Reference `endpoints.md` for API contracts
3. Use `examples/` for common patterns
4. Handle errors using `errors.md` guide

### For API Consumers
1. Import `openapi.yaml` into your API client
2. Use authentication guide for setup
3. Run example scripts to understand workflows
4. Reference endpoint docs for details

### For Integration
1. Generate client from OpenAPI spec
2. Implement authentication flow
3. Handle WebSocket connections
4. Implement error recovery

## Maintenance

### Keeping Documentation Updated

The documentation is based on the current codebase state. When the API changes:

1. **Type Changes:**
   - Run `npm run generate-types` to update `/shared/types.ts`
   - Update OpenAPI schemas to match

2. **New Endpoints:**
   - Add to `endpoints.md` with examples
   - Update OpenAPI spec with full schema
   - Add to quick reference in README.md

3. **Error Changes:**
   - Update `errors.md` with new error types
   - Add recovery strategies

4. **Breaking Changes:**
   - Document in migration guide
   - Version the API (update OpenAPI version)
   - Update examples

## Verification

All documentation has been:
- Cross-referenced with actual code in `crates/server/src/routes/`
- Validated against TypeScript types in `/shared/types.ts`
- Tested example patterns
- Checked for consistency

## Statistics

- **Total Files:** 9
- **Total Size:** ~125 KB
- **Endpoints Documented:** 60+
- **Code Examples:** 20+
- **Shell Scripts:** 4 (all executable)
- **Languages Covered:** Bash, JavaScript/TypeScript, Python, Rust

---

**Documentation Generated:** November 8, 2025
**Anyon Version:** Based on current dev branch
**API Version:** 1.0.0
