# CLAUDE.md

**⚠️ 이 파일은 Claude Code와 AI 어시스턴트 전용입니다.**

일반 개발자 문서는 `/docs` 폴더를 참조하세요.

---

## 📖 Documentation Structure

**이 파일은 간결한 인덱스입니다. 상세한 룰은 `.claude/` 디렉토리에 있습니다:**

```
.claude/
├── CRITICAL-RULES.md          # 🚨 MUST-FOLLOW (type sync, migrations)
├── architecture/
│   ├── frontend.md            # Frontend architecture (33 rules)
│   ├── backend.md             # Backend architecture [TBD]
│   └── code-quality.md        # TDD, Clean Code principles
└── guides/
    ├── commands.md            # Essential commands
    ├── frontend-first.md      # Frontend-first development
    └── development.md         # Development workflows [TBD]
```

**🔍 빠른 찾기:**
- Critical Rules → `.claude/CRITICAL-RULES.md`
- Frontend 작업 → `.claude/architecture/frontend.md`
- 프론트엔드 먼저 개발 → `.claude/guides/frontend-first.md`
- 명령어 → `.claude/guides/commands.md`

---

## 🚨 Critical Rules Summary

**THESE RULES MUST BE FOLLOWED - Violations cause build failures**

### 1. Type Synchronization Chain
```
Rust struct changes → npm run generate-types → Frontend changes
```
- Never manually edit `shared/types.ts`
- Run `npm run generate-types` immediately after modifying Rust structs

### 2. Database Migrations Are Immutable
- Create: `sqlx migrate add description`
- Apply: `sqlx migrate run`
- **NEVER modify or delete existing migrations**

### 3. Development Workflow
1. Design API specs first
2. Implement in any order:
   - **Backend-first**: Rust → Generate types → Frontend
   - **Frontend-first**: Mock API → Frontend → Backend → Connect
3. Sync types when Rust changes: `npm run generate-types`
4. Verify: `npm run check`

### 4. Auto-Generated Code
- `shared/types.ts` is auto-generated - DO NOT EDIT
- Edit Rust source, then regenerate

**📖 Full details:** `.claude/CRITICAL-RULES.md`

---

## 🎨 Frontend Architecture Quick Reference

**Key Principles:**
- **Type Safety First** - Use `shared/types.ts` (auto-generated)
- **State Management**: TanStack Query (server) → Zustand (UI) → useState (local)
- **Component Size** - Max 300 lines
- **Error Boundaries** - Wrap routes with `<PageErrorBoundary>`
- **Code Splitting** - Lazy-load with `React.lazy()`
- **Testing Priority** - API > Hooks > Utils > UI
- **i18n Required** - All user text internationalized
- **Lint** - Max 50 warnings

**📖 Full architecture (33 rules):** `.claude/architecture/frontend.md`

---

## ✨ Code Quality Standards

**TDD: Red → Green → Refactor**

**Clean Code Checklist:**
- [ ] Functions < 20 lines (prefer < 10)
- [ ] Max 3 parameters
- [ ] Cyclomatic complexity < 10
- [ ] Max nesting depth: 3
- [ ] Intention-revealing names
- [ ] No side effects in pure functions
- [ ] Early returns over nested conditions

**📖 Full standards:** `.claude/architecture/code-quality.md`

---

## 🛠️ Essential Commands

### Type Generation
```bash
npm run generate-types           # After Rust changes
npm run generate-types:check     # Verify up to date
```

### Validation
```bash
npm run check                    # TypeScript
cd frontend && npm run lint      # ESLint (max 50 warnings)
cd frontend && npm run test:run  # Tests
cargo test --workspace           # Rust tests
cargo clippy --all              # Rust lint
```

### Database
```bash
sqlx migrate add <name>          # Create migration
sqlx migrate run                 # Apply migrations
```

### Development
```bash
pnpm run dev                     # Frontend + Backend
npm run frontend:dev             # Frontend only
npm run backend:dev              # Backend only
```

**📖 Full commands:** `.claude/guides/commands.md`

---

## 🏗️ Tech Stack

- **Backend**: Rust (Axum, Tokio, SQLx)
- **Frontend**: React 18 + TypeScript + Vite
- **Database**: SQLite + SQLx migrations
- **Type Sharing**: ts-rs (Rust → TypeScript)
- **State**: TanStack Query + Zustand
- **UI**: shadcn/ui + Tailwind CSS
- **Testing**: Vitest + Testing Library

---

## 📁 Project Structure

```
crates/              # Rust backend
├── server/          # Axum HTTP, API routes, MCP
├── db/              # Database, migrations
├── executors/       # AI agent integrations
├── services/        # Business logic
└── utils/           # Shared utilities

frontend/            # React app
├── src/
│   ├── components/  # UI components (by domain)
│   ├── pages/       # Route pages
│   ├── hooks/       # Custom hooks (37)
│   ├── stores/      # Zustand stores (4)
│   ├── contexts/    # React contexts (10)
│   ├── lib/         # API client, utils
│   └── test/        # Test utilities
├── vitest.config.ts
└── package.json

shared/types.ts      # Auto-generated from Rust

.claude/             # Claude Code 전용 문서
docs/                # 일반 사용자 문서 (Mintlify)
```

---

## 🎯 Key Patterns

### 1. Event Streaming (SSE)
- Process logs: `/api/events/processes/:id/logs`
- Use `useEventSourceManager` hook

### 2. Git Worktree Management
- Isolated worktrees per task
- Automatic cleanup

### 3. Executor Pattern
- Pluggable AI agents (Claude, Gemini, etc.)
- Common interface

### 4. MCP Integration
- Anyon as MCP server
- Tools: `list_projects`, `list_tasks`, `create_task`

---

## ✅ Pre-Commit Checklist

- [ ] `npm run check` passes
- [ ] If Rust changed → `npm run generate-types`
- [ ] `cd frontend && npm run lint` passes
- [ ] `cd frontend && npm run test:run` passes
- [ ] `cargo test --workspace` passes
- [ ] No `console.log`
- [ ] No commented code
- [ ] i18n for all user text
- [ ] Error boundaries for new routes
- [ ] Followed architecture rules

**📖 Detailed checklist:** `.claude/architecture/code-quality.md`

---

## 🔗 Quick Links

| What | Where |
|------|-------|
| Critical Rules | `.claude/CRITICAL-RULES.md` |
| Frontend Architecture | `.claude/architecture/frontend.md` |
| Frontend-First Dev | `.claude/guides/frontend-first.md` |
| Code Quality | `.claude/architecture/code-quality.md` |
| Commands | `.claude/guides/commands.md` |

---

**Version:** 2.0.0
**Last Updated:** 2025-11-12
