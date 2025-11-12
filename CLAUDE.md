# CLAUDE.md

**Project guidance for Claude Code and developers**

---

## 📖 Documentation Structure

This file provides a high-level overview. **Detailed rules and guidelines are organized in the `.claude/` directory:**

```
.claude/
├── README.md                      # Navigation guide
├── rules/
│   ├── 01-critical-rules.md       # 🚨 MUST-FOLLOW (type sync, migrations)
│   ├── 02-frontend.md             # Frontend architecture (33 rules)
│   ├── 03-backend.md              # Backend architecture [TBD]
│   └── 04-code-quality.md         # TDD, Clean Code principles
└── workflows/
    ├── commands.md                # Essential commands
    └── development.md             # Development workflows [TBD]
```

**🔍 Finding What You Need:**
- New to the project? → `.claude/README.md`
- Need a specific rule? → See **Quick Reference** below
- Need a command? → `.claude/workflows/commands.md`

---

## 🚨 Critical Rules Summary

**THESE RULES MUST BE FOLLOWED - Violations cause build failures or data loss**

### 1. Type Synchronization Chain
```
Rust struct changes → npm run generate-types → Frontend changes
```
- Never manually edit `shared/types.ts`
- Run `npm run generate-types` immediately after modifying Rust structs with `#[derive(TS)]`

### 2. Database Migrations Are Immutable
- Create: `sqlx migrate add description_here`
- Apply: `sqlx migrate run`
- **NEVER modify or delete existing migrations**

### 3. Development Workflow
1. Design API specs first (Request/Response types, endpoints)
2. Implement in any order (backend-first or frontend-first with mocks)
3. When Rust types change → Sync immediately with `npm run generate-types`
4. Before completing → Run `npm run check`

### 4. Auto-Generated Code
- `shared/types.ts` is auto-generated - DO NOT EDIT
- Edit Rust source instead, then regenerate

**📖 Full details:** `.claude/rules/01-critical-rules.md`

---

## 🎨 Frontend Architecture Quick Reference

**Key Principles:**
- **Type Safety First** - Use types from `shared/types.ts` (auto-generated from Rust)
- **State Management Hierarchy**: TanStack Query (server) → Zustand (UI) → useState (local)
- **Component Size Limit** - Max 300 lines
- **Error Boundaries Required** - Wrap routes with `<PageErrorBoundary>`
- **Code Splitting** - Lazy-load routes with `React.lazy()`
- **Testing Priority** - API client > Hooks > Utils > UI
- **i18n Required** - All user-facing text must be internationalized
- **Lint Standard** - Max 50 warnings

**📖 Full architecture (33 rules):** `.claude/rules/02-frontend.md`

---

## ✨ Code Quality Standards

**TDD Process: Red → Green → Refactor**
1. Write failing test first
2. Write minimal code to pass
3. Refactor while keeping tests green

**Clean Code Checklist:**
- [ ] Functions < 20 lines (prefer < 10)
- [ ] Max 3 parameters (use objects for more)
- [ ] Cyclomatic complexity < 10
- [ ] Max nesting depth: 3 levels
- [ ] Intention-revealing names (no abbreviations)
- [ ] No side effects in pure functions
- [ ] Early returns over nested conditions

**📖 Full standards:** `.claude/rules/04-code-quality.md`

---

## 🛠️ Essential Commands

### Type Generation (After Rust Changes)
```bash
npm run generate-types           # Generate TypeScript from Rust
npm run generate-types:check     # Verify types are up to date
```

### Validation
```bash
npm run check                    # TypeScript type check
cd frontend && npm run lint      # Lint (max 50 warnings)
cd frontend && npm run test:run  # Run tests
cargo test --workspace           # Backend tests
cargo clippy --all              # Rust linting
```

### Database
```bash
sqlx migrate add <description>   # Create migration
sqlx migrate run                 # Apply migrations
```

### Development
```bash
pnpm run dev                     # Start frontend + backend
npm run frontend:dev             # Frontend only
npm run backend:dev              # Backend only
```

**📖 Full command reference:** `.claude/workflows/commands.md`

---

## 🏗️ Tech Stack

- **Backend**: Rust (Axum, Tokio, SQLx)
- **Frontend**: React 18 + TypeScript + Vite
- **Database**: SQLite with SQLx migrations
- **Type Sharing**: ts-rs (Rust → TypeScript auto-generation)
- **State Management**: TanStack Query + Zustand
- **UI Components**: shadcn/ui + Tailwind CSS
- **Testing**: Vitest + Testing Library (frontend), Cargo test (backend)

---

## 📁 Project Structure

```
crates/
├── server/         # Axum HTTP server, API routes, MCP server
├── db/             # Database models, migrations, SQLx queries
├── executors/      # AI coding agent integrations
├── services/       # Business logic (GitHub, auth, git operations)
├── local-deployment/ # Local deployment logic
└── utils/          # Shared utilities

frontend/
├── src/
│   ├── components/ # React components (by domain)
│   ├── pages/      # Route pages
│   ├── hooks/      # Custom React hooks (37 hooks)
│   ├── stores/     # Zustand stores (4 stores)
│   ├── contexts/   # React contexts (10 contexts)
│   ├── lib/        # API client, utilities
│   └── test/       # Test setup and utilities
├── vitest.config.ts # Test configuration
└── package.json

shared/types.ts     # Auto-generated TypeScript types from Rust

.claude/            # Documentation (rules, workflows)
```

---

## 🎯 Key Architectural Patterns

### 1. Event Streaming (SSE)
- Real-time process logs: `/api/events/processes/:id/logs`
- Task diffs: `/api/events/task-attempts/:id/diff`
- Use `useEventSourceManager` hook

### 2. Git Worktree Management
- Each task execution gets isolated git worktree
- Automatic cleanup of orphaned worktrees
- Managed by `WorktreeManager` service

### 3. Executor Pattern
- Pluggable AI agent executors (Claude, Gemini, etc.)
- Common interface for all executors
- Actions: `coding_agent_initial`, `coding_agent_follow_up`, `script`

### 4. MCP Integration
- Anyon acts as MCP server
- Tools: `list_projects`, `list_tasks`, `create_task`, `update_task`
- AI agents can manage tasks via MCP protocol

---

## ✅ Pre-Commit Checklist

Before every commit, verify:

- [ ] `npm run check` passes (TypeScript)
- [ ] If Rust structs changed → Ran `npm run generate-types`
- [ ] `cd frontend && npm run lint` passes (max 50 warnings)
- [ ] `cd frontend && npm run test:run` passes (if tests exist)
- [ ] `cargo test --workspace` passes
- [ ] No `console.log` statements
- [ ] No commented-out code
- [ ] All user-facing text is internationalized
- [ ] Error boundaries added for new routes
- [ ] Followed architecture rules

**📖 Detailed checklist:** `.claude/rules/04-code-quality.md`

---

## 🆘 When in Doubt

1. **Check existing patterns** - Look for similar code in the codebase
2. **Consult the rules** - See `.claude/rules/` for detailed guidelines
3. **Follow TypeScript errors** - They usually point to the right solution
4. **Keep it simple** - Don't over-engineer
5. **Ask questions** - Better to clarify than make wrong assumptions

---

## 🔗 Quick Links

| Topic | Document |
|-------|----------|
| Critical Rules | `.claude/rules/01-critical-rules.md` |
| Frontend Architecture | `.claude/rules/02-frontend.md` |
| Code Quality | `.claude/rules/04-code-quality.md` |
| Commands | `.claude/workflows/commands.md` |
| Navigation | `.claude/README.md` |

---

**Version:** 2.0.0 (Structured)
**Last Updated:** 2025-11-12
