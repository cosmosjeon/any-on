# Codebase Structure

## Root Directory
```
any-on/
├── crates/              # Rust backend workspace
├── frontend/            # React TypeScript frontend
├── shared/              # Auto-generated TypeScript types
├── docs/                # User documentation (Mintlify)
├── .claude/             # Claude Code specific instructions
├── scripts/             # Build and deployment scripts
├── dev_assets/          # Development database
├── dev_assets_seed/     # Fresh DB template
├── migrations/          # SQLx database migrations
├── data/                # Runtime data directory
├── logs/                # Application logs
└── npx-cli/             # NPX package distribution
```

## Backend Structure (crates/)
```
crates/
├── server/              # Main HTTP server
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── routes/      # API route handlers
│   │   ├── mcp/         # MCP server implementation
│   │   ├── websocket/   # WebSocket handlers
│   │   └── middleware/  # Axum middleware
│   └── Cargo.toml
│
├── db/                  # Database layer
│   ├── src/
│   │   ├── lib.rs
│   │   ├── models/      # SQLite models
│   │   ├── repos/       # Repository pattern (CRUD)
│   │   └── migrations/  # SQLx migrations
│   └── Cargo.toml
│
├── executors/           # AI agent integrations
│   ├── src/
│   │   ├── lib.rs
│   │   ├── claude.rs    # Claude Code executor
│   │   ├── gemini.rs    # Gemini CLI executor
│   │   └── common.rs    # Shared executor traits
│   └── Cargo.toml
│
├── services/            # Business logic layer
│   ├── src/
│   │   ├── task_service.rs
│   │   ├── project_service.rs
│   │   ├── worktree_service.rs
│   │   └── process_service.rs
│   └── Cargo.toml
│
└── utils/               # Shared utilities
    ├── src/
    │   ├── git.rs       # Git operations
    │   ├── fs.rs        # File system utilities
    │   └── crypto.rs    # Encryption/hashing
    └── Cargo.toml
```

## Frontend Structure (frontend/)
```
frontend/
├── src/
│   ├── main.tsx         # Entry point
│   ├── App.tsx          # Root component
│   │
│   ├── components/      # UI components (by domain)
│   │   ├── layout/      # Layout components (Navbar, Sidebar)
│   │   ├── tasks/       # Task-related components
│   │   ├── projects/    # Project-related components
│   │   ├── processes/   # Process/log components
│   │   └── ui/          # shadcn/ui primitives
│   │
│   ├── pages/           # Route pages
│   │   ├── TasksPage.tsx
│   │   ├── ProjectsPage.tsx
│   │   ├── SettingsPage.tsx
│   │   └── kanban/      # Kanban board pages
│   │
│   ├── hooks/           # Custom React hooks (37 hooks)
│   │   ├── useTaskService.ts
│   │   ├── useProjectService.ts
│   │   ├── useEventSourceManager.ts
│   │   └── useWebSocketManager.ts
│   │
│   ├── stores/          # Zustand stores (4 stores)
│   │   ├── uiStore.ts   # UI state (sidebar, modals)
│   │   ├── notificationStore.ts
│   │   └── kanbanStore.ts
│   │
│   ├── contexts/        # React contexts (10 contexts)
│   │   ├── AuthContext.tsx
│   │   ├── ThemeContext.tsx
│   │   ├── I18nContext.tsx
│   │   └── SettingsContext.tsx
│   │
│   ├── lib/             # Core libraries
│   │   ├── api.ts       # API client (fetch wrapper)
│   │   ├── queryClient.ts  # TanStack Query config
│   │   ├── router.tsx   # React Router setup
│   │   └── utils.ts     # Utility functions
│   │
│   ├── styles/          # Global styles
│   │   ├── index.css    # Tailwind imports
│   │   └── globals.css  # Global CSS
│   │
│   ├── test/            # Test utilities
│   │   ├── setup.ts     # Vitest setup
│   │   ├── mocks/       # Mock data
│   │   └── utils.tsx    # Test helpers
│   │
│   └── locales/         # i18n translations
│       ├── en/          # English
│       ├── ko/          # Korean
│       └── i18n.ts      # i18next config
│
├── public/              # Static assets
│   ├── favicon.svg
│   ├── anyon-logo.svg
│   └── anyon-logo-dark.svg
│
├── vitest.config.ts     # Vitest configuration
├── vite.config.ts       # Vite configuration
├── tailwind.config.js   # Tailwind CSS config
├── tsconfig.json        # TypeScript config
└── package.json
```

## Shared Types (shared/)
```
shared/
└── types.ts             # AUTO-GENERATED from Rust (DO NOT EDIT)
```

## Documentation (.claude/)
```
.claude/
├── CRITICAL-RULES.md    # 🚨 Must-follow rules
├── architecture/
│   ├── frontend.md      # Frontend architecture (33 rules)
│   ├── backend.md       # Backend architecture [TBD]
│   └── code-quality.md  # TDD, Clean Code principles
└── guides/
    ├── commands.md      # Essential commands
    ├── frontend-first.md # Frontend-first development
    └── development.md   # Development workflows [TBD]
```

## Key File Patterns

### Auto-Generated Files (DO NOT EDIT)
- `shared/types.ts` - Generated by ts-rs from Rust structs

### Configuration Files
- `Cargo.toml` (root) - Rust workspace
- `package.json` (root) - NPM scripts, monorepo
- `frontend/package.json` - Frontend dependencies
- `.env` - Runtime environment variables
- `.env.cloud` - Cloud deployment config
- `rust-toolchain.toml` - Rust version pinning

### Database
- `migrations/*.sql` - SQLx migrations (IMMUTABLE)
- `dev_assets/.dev.db` - Development database
- `dev_assets_seed/.dev.db` - Fresh DB template

## Important Locations

### Entry Points
- Backend: `crates/server/src/main.rs`
- Frontend: `frontend/src/main.tsx`
- Type Generator: `crates/generate_types/src/main.rs`

### API Routes
- Defined in: `crates/server/src/routes/`
- Example: `crates/server/src/routes/tasks.rs`

### Models & Types
- Rust models: `crates/db/src/models/`
- TypeScript types: `shared/types.ts` (auto-generated)

### Tests
- Frontend tests: `frontend/src/**/*.test.ts(x)`
- Rust tests: `crates/**/src/**/*.rs` (inline `#[cfg(test)]`)

## Git Worktree Structure
```
data/
└── projects/
    └── <project-id>/
        ├── main/        # Main working directory
        └── worktrees/   # Task-specific worktrees
            ├── task-1/
            ├── task-2/
            └── ...
```

Worktrees are automatically created for each task and cleaned up when tasks complete.
