# Suggested Commands (macOS/Darwin)

## Development

### Start Development Environment
```bash
pnpm run dev                      # Start both frontend + backend concurrently
npm run frontend:dev              # Frontend only (port from .dev-ports.json)
npm run backend:dev               # Backend only (auto-watch with cargo-watch)
```

### Type Generation (CRITICAL)
```bash
npm run generate-types            # Generate TypeScript types from Rust structs
npm run generate-types:check      # Verify types are up-to-date (CI check)
```

## Validation & Quality

### Type Checking
```bash
npm run check                     # TypeScript type check (frontend + backend)
cd frontend && npm run check      # Frontend only
npm run backend:check             # Backend only (cargo check)
```

### Linting
```bash
npm run lint                      # Lint both frontend + backend
cd frontend && npm run lint       # ESLint (max 50 warnings)
cd frontend && npm run lint:fix   # Auto-fix ESLint issues
npm run backend:lint              # Clippy (Rust linter)
```

### Formatting
```bash
npm run format                    # Format both frontend + backend
cd frontend && npm run format     # Prettier
cargo fmt --all                   # rustfmt
```

### Testing
```bash
cd frontend && npm run test       # Interactive test mode (Vitest)
cd frontend && npm run test:run   # Run all tests once
cd frontend && npm run test:ui    # Open Vitest UI
cd frontend && npm run test:coverage  # Coverage report
cargo test --workspace            # Rust tests
```

## Database

### Migrations
```bash
sqlx migrate add <description>    # Create new migration file
sqlx migrate run                  # Apply pending migrations
sqlx migrate revert               # Revert last migration (dev only)
sqlx migrate info                 # Show migration status
```

### Development Database
```bash
npm run prepare-db                # Copy fresh DB from dev_assets_seed
```

## Build & Package

### Build
```bash
cd frontend && pnpm build         # Build frontend for production
cargo build --release             # Build backend (release mode)
npm run build:npx                 # Build full NPX package
```

### Test NPX Package
```bash
npm run test:npm                  # Test the NPX package locally
```

## Utility Commands (macOS)

### File Operations
```bash
ls -la                            # List files (including hidden)
find . -name "*.tsx"              # Find files by pattern
grep -r "pattern" src/            # Search in files
cat filename                      # Display file contents
head -n 20 filename               # First 20 lines
tail -n 20 filename               # Last 20 lines
```

### Git
```bash
git status                        # Check working tree status
git diff                          # Show changes
git add .                         # Stage all changes
git commit -m "message"           # Commit with message
git log --oneline -10             # Show last 10 commits
```

### Process Management
```bash
ps aux | grep "node"              # Find Node processes
lsof -i :3000                     # Check what's using port 3000
kill -9 <PID>                     # Kill process by PID
```

### Development Helpers
```bash
pnpm i                            # Install dependencies
pnpm i <package>                  # Add package
cargo install cargo-watch         # Install Rust auto-watch
cargo install sqlx-cli            # Install SQLx CLI
```

## Pre-Commit Workflow

**Before every commit:**
```bash
# 1. If Rust structs changed
npm run generate-types

# 2. Type check
npm run check

# 3. Lint
npm run lint

# 4. Test
cd frontend && npm run test:run
cargo test --workspace

# 5. Format
npm run format
```

## Common Workflows

### After Rust Struct Changes
```bash
npm run generate-types            # MUST DO FIRST
npm run check                     # Verify TypeScript types
cd frontend && npm run lint       # Check for any issues
```

### Adding a New Feature
```bash
# 1. Create feature branch
git checkout -b feat/feature-name

# 2. Develop (iterate)
pnpm run dev                      # Run dev server

# 3. Before commit
npm run generate-types            # If Rust changed
npm run check && npm run lint
cd frontend && npm run test:run

# 4. Commit
git add .
git commit -m "feat: description"
```

### Database Schema Change
```bash
# 1. Create migration
sqlx migrate add description_here

# 2. Edit migration SQL file in migrations/

# 3. Apply migration
sqlx migrate run

# 4. Update Rust models in crates/db/src/models/

# 5. If models have #[derive(TS)]
npm run generate-types
```

## Environment Variables

### Development (.env)
```bash
BACKEND_PORT=0                    # 0 = auto-assign
FRONTEND_PORT=3000
HOST=127.0.0.1
DISABLE_WORKTREE_ORPHAN_CLEANUP=1 # For debugging
```

### Build-time
```bash
GITHUB_CLIENT_ID=<id>             # Custom GitHub OAuth
POSTHOG_API_KEY=<key>             # Analytics
POSTHOG_API_ENDPOINT=<url>        # Analytics endpoint
```
