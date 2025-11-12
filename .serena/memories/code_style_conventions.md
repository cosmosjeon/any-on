# Code Style & Conventions

## Critical Rules (Build-Breaking)

### 1. Type Synchronization Chain
- **MUST** run `npm run generate-types` after ANY Rust struct change with `#[derive(TS)]`
- **NEVER** manually edit `shared/types.ts` - it's auto-generated
- Flow: Rust struct → generate-types → Frontend changes

### 2. Database Migrations
- **IMMUTABLE**: Never modify or delete existing migrations
- Format: `YYYYMMDDHHMMSS_description.sql`
- Create: `sqlx migrate add description`
- Apply: `sqlx migrate run`

## Frontend Code Style

### Naming Conventions
- **Components**: PascalCase (`TaskCard`, `UserProfile`)
- **Functions**: camelCase, verb-based (`createTask`, `fetchUser`)
- **Booleans**: Prefix with is/has/can/should (`isLoading`, `hasError`)
- **Constants**: UPPER_SNAKE_CASE (`MAX_RETRY_ATTEMPTS`, `API_BASE_URL`)
- **Files**: kebab-case for utilities, PascalCase for components

### Function Guidelines
- **Max 20 lines** (prefer < 10 lines)
- **Max 3 parameters** (use objects for more)
- **Single responsibility** - one function = one thing
- **Early returns** over nested conditions
- **No side effects** in pure functions
- **Cyclomatic complexity < 10**
- **Max nesting depth: 3**

### TypeScript Best Practices
- **Explicit boolean conditions**: `{items.length > 0 && <List />}`
- **Nullish comparisons**: `value != null ? value : default`
- **Avoid nested ternaries** - use if/else or functions
- **No `any` types** without justification
- **Intention-revealing names** - no abbreviations

### Component Structure
- **Max 300 lines** per component
- **Extract hooks** for complex logic
- **Error boundaries** for route-level components
- **Code splitting** with `React.lazy()` for routes
- **Feature-based organization** over type-based

### State Management Hierarchy
1. **TanStack Query** - Server state, caching, invalidation
2. **Zustand** - UI state (sidebar, modals, notifications)
3. **Context** - Cross-cutting concerns (auth, theme, i18n)
4. **useState** - Local component state

### Testing
- **Priority**: API > Hooks > Utils > UI
- **TDD**: Red → Green → Refactor
- **AAA Pattern**: Arrange → Act → Assert
- **FIRST Principles**: Fast, Independent, Repeatable, Self-validating, Timely

### i18n
- **ALL user-facing text** must use i18next
- Keys format: `feature.component.text` (e.g., `tasks.card.title`)
- No hardcoded English strings

## Clean Code Principles
- **DRY** - Don't Repeat Yourself
- **KISS** - Keep It Simple, Stupid
- **YAGNI** - You Aren't Gonna Need It
- **SOLID** - Apply all five principles
- **Boy Scout Rule** - Leave code cleaner than you found it

## Error Handling
- **Fail fast** with clear error messages
- **Use exceptions** over error codes
- **Handle at appropriate levels** (API boundary vs UI)
- **Log with context** (include taskId, timestamp, etc.)
- **Never catch generic exceptions** without re-throwing

## Code Review Standards
- No `console.log` statements (use proper logging)
- No commented-out code
- No TODOs without issue tickets
- All functions < 20 lines
- Complexity < 10
- Clear, intention-revealing names
- Tests pass
- Linting passes (max 50 warnings)
