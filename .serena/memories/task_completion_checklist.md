# Task Completion Checklist

## Pre-Commit Checklist

### Type Synchronization (CRITICAL)
- [ ] If Rust structs changed → Run `npm run generate-types`
- [ ] If types changed → Verify with `npm run generate-types:check`

### Type Checking
- [ ] Run `npm run check` (TypeScript type check)
- [ ] No TypeScript errors in IDE
- [ ] No new `any` types without justification

### Linting
- [ ] Run `cd frontend && npm run lint`
- [ ] Max 50 warnings (enforced)
- [ ] No ESLint errors
- [ ] Run `cargo clippy --workspace` for Rust

### Testing
- [ ] Run `cd frontend && npm run test:run`
- [ ] All tests pass
- [ ] Run `cargo test --workspace` for Rust tests
- [ ] Test coverage is acceptable

### Code Quality
- [ ] No `console.log` statements
- [ ] No commented-out code
- [ ] No TODOs without issue tickets
- [ ] All functions < 20 lines
- [ ] Cyclomatic complexity < 10
- [ ] Max nesting depth: 3
- [ ] Clear, intention-revealing names
- [ ] No abbreviations in variable names

### Frontend-Specific
- [ ] All user-facing text uses i18next (no hardcoded strings)
- [ ] Error boundaries added for new routes
- [ ] Components < 300 lines
- [ ] Code splitting used for new routes (`React.lazy()`)
- [ ] Followed state management hierarchy (TanStack Query → Zustand → Context → useState)

### Backend-Specific
- [ ] If database schema changed → Created migration with `sqlx migrate add`
- [ ] Run `sqlx migrate run` to apply migrations
- [ ] Updated models in `crates/db/src/models/`
- [ ] NEVER modified existing migration files

### Architecture Rules
- [ ] Followed CRITICAL-RULES.md (`.claude/CRITICAL-RULES.md`)
- [ ] Followed frontend architecture (`.claude/architecture/frontend.md`)
- [ ] Followed code quality standards (`.claude/architecture/code-quality.md`)
- [ ] No violations of type sync chain
- [ ] No violations of database migration immutability

### Documentation
- [ ] Updated relevant documentation if APIs changed
- [ ] Added comments for complex logic
- [ ] Updated README if new features added

## Git Commit Standards

### Commit Message Format
```
<type>: <description>

[optional body]

[optional footer]
```

### Types
- `feat:` - New feature
- `fix:` - Bug fix
- `refactor:` - Code restructuring (no behavior change)
- `test:` - Adding or updating tests
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, no logic change)
- `chore:` - Build, tooling, dependencies
- `perf:` - Performance improvements

### Examples
```bash
git commit -m "feat: add task filtering by status"
git commit -m "fix: resolve type mismatch in TaskCard component"
git commit -m "refactor: extract task validation logic into hook"
```

## Post-Commit Verification

### CI Checks (Automated)
- [ ] TypeScript compilation passes
- [ ] All tests pass
- [ ] Linting passes (max 50 warnings)
- [ ] Type generation check passes (`generate-types:check`)

### Manual Verification
- [ ] Feature works in development mode (`pnpm run dev`)
- [ ] No console errors in browser DevTools
- [ ] No network errors (check Network tab)
- [ ] UI is responsive (test different screen sizes)
- [ ] Works in production build (`cd frontend && pnpm build`)

## When Task is Complete

### 1. Final Quality Check
```bash
# Run all validation
npm run check
npm run lint
cd frontend && npm run test:run
cargo test --workspace
```

### 2. Format Code
```bash
npm run format
```

### 3. Stage & Commit
```bash
git add .
git commit -m "feat: description of changes"
```

### 4. Push & Create PR
```bash
git push origin feat/branch-name
# Open PR on GitHub
```

### 5. PR Checklist
- [ ] Title follows commit message format
- [ ] Description explains what and why
- [ ] Screenshots/videos for UI changes
- [ ] All CI checks passing
- [ ] Self-review completed
- [ ] Ready for code review

## Critical Don'ts

❌ **NEVER** manually edit `shared/types.ts`
❌ **NEVER** modify or delete existing migration files
❌ **NEVER** skip `npm run generate-types` after Rust changes
❌ **NEVER** commit with TypeScript errors
❌ **NEVER** commit with more than 50 ESLint warnings
❌ **NEVER** commit `console.log` statements
❌ **NEVER** commit commented-out code
❌ **NEVER** commit with failing tests

## Quick Command Reference

```bash
# Full validation suite
npm run check && npm run lint && cd frontend && npm run test:run && cd .. && cargo test --workspace

# Generate types + validate
npm run generate-types && npm run check

# Format + lint
npm run format && npm run lint

# Build everything
cd frontend && pnpm build && cd .. && cargo build --release
```

## Debugging Checklist

If something is broken:

1. **Type Issues**
   ```bash
   npm run generate-types    # Regenerate types
   npm run check             # Check for errors
   ```

2. **Database Issues**
   ```bash
   sqlx migrate info         # Check migration status
   npm run prepare-db        # Reset to fresh DB
   ```

3. **Dependency Issues**
   ```bash
   pnpm i                    # Reinstall dependencies
   cargo clean && cargo build # Rebuild Rust
   ```

4. **Port Conflicts**
   ```bash
   lsof -i :3000             # Check what's using port
   kill -9 <PID>             # Kill process
   ```

5. **Git Worktree Issues**
   ```bash
   git worktree list         # List worktrees
   git worktree prune        # Clean up stale worktrees
   ```
