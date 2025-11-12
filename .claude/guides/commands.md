# Essential Commands Reference

Quick reference for common development commands.

---

## Development Servers

```bash
# Start both frontend + backend
pnpm run dev

# Frontend only (port 3000)
npm run frontend:dev

# Backend only (port auto-assigned)
npm run backend:dev

# Preview production build
npm run preview
```

---

## Type Generation

```bash
# Generate TypeScript types from Rust
npm run generate-types

# Check if types are up to date
npm run generate-types:check
```

⚠️ **Run this immediately after modifying any Rust struct with `#[derive(TS)]`**

---

## Testing & Validation

### Frontend

```bash
# TypeScript type check
cd frontend && npm run check
# or from root
npm run check

# Linting (max 50 warnings)
cd frontend && npm run lint

# Auto-fix linting issues
cd frontend && npm run lint:fix

# i18n linting (no warnings allowed)
cd frontend && npm run lint:i18n

# Code formatting check
cd frontend && npm run format:check

# Auto-format code
cd frontend && npm run format

# Run tests
cd frontend && npm run test         # Watch mode
cd frontend && npm run test:ui      # UI mode
cd frontend && npm run test:run     # Single run
cd frontend && npm run test:coverage # With coverage
```

### Backend

```bash
# Run all tests
cargo test --workspace

# Test specific crate
cargo test -p server
cargo test -p db

# Run specific test
cargo test test_create_task

# Check formatting
cargo fmt --all -- --check

# Auto-format
cargo fmt --all

# Linting (Clippy)
cargo clippy --all --all-targets --all-features -- -D warnings
```

---

## Database Operations

```bash
# Create a new migration
sqlx migrate add add_task_priority

# Apply migrations
sqlx migrate run

# Create database (if needed)
sqlx database create

# Revert last migration (use with caution!)
sqlx migrate revert
```

⚠️ **Never modify existing migrations!** Create new ones instead.

---

## Build

```bash
# Build frontend
cd frontend && npm run build

# Build backend
cargo build --release

# Build NPM package (production)
./build-npm-package.sh
```

---

## Git Workflow

```bash
# Create feature branch
git checkout -b feat/task-kanban-view

# Stage changes
git add .

# Commit with conventional format
git commit -m "feat(tasks): add kanban board view"

# Push to remote
git push -u origin feat/task-kanban-view

# Create pull request (using GitHub CLI)
gh pr create --title "Add kanban board view" --body "..."
```

---

## Common Workflows

### Adding a New API Endpoint

```bash
# 1. Design API spec (Request/Response types)

# 2. Add Rust structs with TS export
# Edit crates/server/src/routes/tasks.rs

# 3. Generate TypeScript types
npm run generate-types

# 4. Implement frontend
# Edit frontend/src/lib/api.ts

# 5. Verify
npm run check
cargo test
```

### Modifying Database Schema

```bash
# 1. Create migration
sqlx migrate add update_task_schema

# 2. Write SQL (edit migrations/YYYYMMDDHHMMSS_update_task_schema.sql)

# 3. Apply migration
sqlx migrate run

# 4. Update Rust models (crates/db/src/models/task.rs)

# 5. Update types
npm run generate-types

# 6. Update frontend if needed
```

### Fixing TypeScript Errors After Rust Changes

```bash
# 1. Regenerate types
npm run generate-types

# 2. Check errors
npm run check

# 3. Fix errors in frontend code

# 4. Verify
npm run check
npm run lint
```

---

## Troubleshooting

### "Type X not found" in Frontend
```bash
# Regenerate types
npm run generate-types

# Clear node_modules cache (if needed)
rm -rf node_modules/.vite
```

### Migration Conflicts
```bash
# Check migration status
sqlx migrate info

# If stuck, manually check migrations table in SQLite
sqlite3 ~/.config/anyon/db.sqlite "SELECT * FROM _sqlx_migrations;"
```

### Build Errors
```bash
# Clean and rebuild
cargo clean
cd frontend && rm -rf dist node_modules/.vite
pnpm install
pnpm run dev
```

---

## Quick Checks Before Commit

```bash
# Run all checks
npm run check                    # TypeScript
cd frontend && npm run lint      # ESLint
cd frontend && npm run format:check  # Prettier
cargo test --workspace           # Rust tests
cargo clippy --all              # Rust linting
```

Or create an alias in your shell:

```bash
# Add to ~/.zshrc or ~/.bashrc
alias precommit='npm run check && cd frontend && npm run lint && npm run format:check && cd .. && cargo test --workspace && cargo clippy --all'
```

Then just run: `precommit`
