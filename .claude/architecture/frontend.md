# Frontend Architecture Rules

This document provides architectural guidelines and rules for the ANYON frontend codebase. **All developers and AI assistants (including Claude Code) MUST follow these rules when working on the frontend.**

## 🚨 CRITICAL: Core Principles

### 1. Type Safety First
**Rule:** Always use explicit types. Avoid `any` unless absolutely necessary.

**DO:**
```typescript
interface UserData {
  id: string;
  name: string;
  email: string;
}

function getUser(id: string): Promise<UserData> {
  return api.get<UserData>(`/users/${id}`);
}
```

**DON'T:**
```typescript
// ❌ Avoid any
function getUser(id: any): any {
  return api.get(`/users/${id}`);
}
```

**Exception:** RJSF (React JSON Schema Form) library requires `any` for form data due to its dynamic nature. In this case, use `// eslint-disable-next-line @typescript-eslint/no-explicit-any` with a comment explaining why.

### 2. Use Auto-Generated Types from Rust
**Rule:** Never manually define types that exist in `shared/types.ts`. This file is auto-generated from Rust structs.

**Workflow:**
1. Modify Rust struct → 2. Run `npm run generate-types` → 3. Import types in frontend

**DO:**
```typescript
import { Task, TaskStatus, TaskAttempt } from 'shared/types';

function createTask(task: Task): Promise<Task> {
  return tasksApi.create(task);
}
```

**DON'T:**
```typescript
// ❌ Don't duplicate types
interface Task {
  id: string;
  title: string;
  // ... duplicating what's in shared/types.ts
}
```

### 3. Component Structure Rules
**Rule:** Follow the established component hierarchy and separation of concerns.

**Component Organization:**
```
components/
├── ui/              # Reusable UI primitives (shadcn/ui)
├── tasks/           # Task-specific domain components
├── projects/        # Project-specific domain components
├── dialogs/         # Modal dialogs
│   ├── auth/       # Authentication dialogs
│   ├── tasks/      # Task-related dialogs
│   ├── settings/   # Settings dialogs
│   └── global/     # Global dialogs (onboarding, etc.)
├── panels/          # Panel components (DiffsPanel, TaskPanel)
├── layout/          # Layout components (NormalLayout, etc.)
└── [feature]/       # Feature-specific components
```

**Size Limits:**
- Max 300 lines per component file
- Max 150 lines per function
- If exceeded, split into smaller components

**DO:**
```typescript
// TaskCard.tsx - Small, focused component
export function TaskCard({ task }: { task: Task }) {
  return (
    <Card>
      <TaskHeader task={task} />
      <TaskBody task={task} />
      <TaskActions task={task} />
    </Card>
  );
}
```

**DON'T:**
```typescript
// ❌ 500+ line monolithic component
export function TaskCard({ task }: { task: Task }) {
  // ... 500 lines of code mixing UI, logic, and state
}
```

---

## 🏗️ State Management Architecture

### 4. State Management Decision Tree

**Follow this hierarchy when choosing state management:**

```
1. Does it need to be shared across routes?
   ├─ YES → Use TanStack Query (server state) or Zustand (client state)
   └─ NO → Continue to 2

2. Is it complex UI state used by multiple components?
   ├─ YES → Use Zustand store (e.g., useUIStore)
   └─ NO → Continue to 3

3. Is it local component state?
   ├─ YES → Use useState/useReducer
   └─ NO → Continue to 4

4. Is it derived/computed state?
   └─ Use useMemo
```

### 5. Zustand for UI State
**Rule:** Use Zustand for complex UI state that doesn't fit in React state.

**DO:**
```typescript
// stores/useUIStore.ts
export const useUIStore = create<UIState>((set) => ({
  selectedProcessId: null,
  setSelectedProcessId: (id) => set({ selectedProcessId: id }),
}));

// In component
const { selectedProcessId, setSelectedProcessId } = useUIStore();
```

**Available Zustand Stores:**
- `useUIStore` - Unified UI state (process selection, tabs, retry states, etc.)
- `useDiffViewStore` - Diff view preferences (mode, whitespace)
- `useExpandableStore` - Expandable sections state
- `useTaskDetailsUiStore` - Task details UI state

### 6. TanStack Query for Server State
**Rule:** All server data MUST be managed through TanStack Query, not useState.

**DO:**
```typescript
// Custom hook for server data
export function useProject(projectId: string) {
  return useQuery({
    queryKey: ['project', projectId],
    queryFn: () => projectsApi.getById(projectId),
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

// In component
const { data: project, isLoading } = useProject(projectId);
```

**DON'T:**
```typescript
// ❌ Managing server state with useState
const [project, setProject] = useState(null);
useEffect(() => {
  projectsApi.getById(projectId).then(setProject);
}, [projectId]);
```

### 7. Context Usage Rules
**Rule:** Only use Context for truly global, infrequently-changing data.

**Acceptable Context Use Cases:**
- Theme (ThemeProvider)
- i18n (I18nextProvider)
- User configuration (UserSystemProvider)
- Current project info (ProjectProvider)
- Search state tied to routes (SearchProvider)

**DON'T use Context for:**
- ❌ Frequently changing UI state → Use Zustand
- ❌ Server data → Use TanStack Query
- ❌ Form state → Use local useState or form libraries

---

## 🎨 Component Patterns

### 8. Error Boundaries
**Rule:** Wrap all route-level components with error boundaries.

**DO:**
```typescript
// App.tsx
<Route
  path="/projects/:projectId/tasks"
  element={
    <PageErrorBoundary>
      <ProjectTasks />
    </PageErrorBoundary>
  }
/>
```

**Available Error Boundaries:**
- `PageErrorBoundary` - Full page error UI (use for routes)
- `ComponentErrorBoundary` - Inline error UI (use for components)

### 9. Code Splitting
**Rule:** Use React.lazy() for all route-level components.

**DO:**
```typescript
// App.tsx
const Projects = lazy(() =>
  import('@/pages/projects').then(m => ({ default: m.Projects }))
);

<Suspense fallback={<Loader />}>
  <Route path="/projects" element={<Projects />} />
</Suspense>
```

**DON'T:**
```typescript
// ❌ Direct import for route components
import { Projects } from '@/pages/projects';
```

### 10. Custom Hooks Pattern
**Rule:** Extract complex logic into custom hooks.

**DO:**
```typescript
// hooks/useTaskExecution.ts
export function useTaskExecution(taskId: string) {
  const [isRunning, setIsRunning] = useState(false);

  const execute = useCallback(async () => {
    setIsRunning(true);
    try {
      await tasksApi.execute(taskId);
    } finally {
      setIsRunning(false);
    }
  }, [taskId]);

  return { execute, isRunning };
}
```

**Existing Key Hooks:**
- `useConversationHistory` - Real-time SSE conversation updates
- `useAttemptExecution` - Task attempt execution state
- `useEventSourceManager` - Server-Sent Events management
- `useProject` - Current project context

### 11. Prop Interface Naming
**Rule:** Use clear, explicit names for prop interfaces.

**DO:**
```typescript
interface TaskCardProps {
  task: Task;
  onEdit: (task: Task) => void;
  isSelected?: boolean;
}

export function TaskCard({ task, onEdit, isSelected }: TaskCardProps) {
  // ...
}
```

**DON'T:**
```typescript
// ❌ Generic "Props" name is ambiguous in large files
interface Props {
  task: Task;
  onEdit: (task: Task) => void;
}
```

---

## 📦 API & Data Fetching

### 12. API Client Structure
**Rule:** All API calls MUST go through the centralized API client in `lib/api.ts`.

**Structure:**
```typescript
// lib/api.ts
export const tasksApi = {
  getAll: async (projectId: string): Promise<Task[]> => {...},
  getById: async (taskId: string): Promise<Task> => {...},
  create: async (data: CreateTask): Promise<Task> => {...},
  update: async (taskId: string, data: UpdateTask): Promise<Task> => {...},
  delete: async (taskId: string): Promise<void> => {...},
};
```

**DO:**
```typescript
import { tasksApi } from '@/lib/api';

const tasks = await tasksApi.getAll(projectId);
```

**DON'T:**
```typescript
// ❌ Direct fetch in components
const response = await fetch(`/api/projects/${projectId}/tasks`);
const tasks = await response.json();
```

### 13. Error Handling Pattern
**Rule:** Use the Result<T, E> type for functions that can fail.

**DO:**
```typescript
import { Result, Ok, Err } from '@/lib/api';

async function saveTask(task: Task): Promise<Result<Task, string>> {
  try {
    const saved = await tasksApi.create(task);
    return Ok(saved);
  } catch (error) {
    return Err(error.message);
  }
}

// In component
const result = await saveTask(newTask);
if (result.ok) {
  console.log('Saved:', result.value);
} else {
  console.error('Error:', result.error);
}
```

### 14. Real-Time Data (SSE)
**Rule:** Use `useEventSourceManager` for Server-Sent Events.

**DO:**
```typescript
import { useEventSourceManager } from '@/hooks/useEventSourceManager';

const { state, startStream, stopStream } = useEventSourceManager({
  url: `/api/events/processes/${processId}/logs`,
  onMessage: (data) => {
    // Handle streaming data
  },
});
```

---

## 🎯 Performance Optimization

### 15. Memoization Rules
**Rule:** Use memoization strategically, not everywhere.

**When to use useMemo:**
- ✅ Expensive computations (sorting, filtering large arrays)
- ✅ Creating objects/arrays passed to child components as props
- ❌ Simple calculations (addition, string concatenation)
- ❌ Primitive values

**When to use useCallback:**
- ✅ Callbacks passed to memoized child components
- ✅ Callbacks used in dependency arrays
- ❌ Event handlers for simple operations

**DO:**
```typescript
// Expensive computation
const sortedTasks = useMemo(() =>
  tasks.sort((a, b) => a.priority - b.priority),
  [tasks]
);

// Callback for memoized child
const handleEdit = useCallback((task: Task) => {
  setEditingTask(task);
}, []);
```

**DON'T:**
```typescript
// ❌ Over-memoization
const name = useMemo(() => `${firstName} ${lastName}`, [firstName, lastName]);
const doubled = useMemo(() => value * 2, [value]);
```

### 16. List Rendering
**Rule:** Use virtualization for large lists (50+ items).

**DO:**
```typescript
import { Virtuoso } from 'react-virtuoso';

<Virtuoso
  data={tasks}
  itemContent={(index, task) => <TaskCard task={task} />}
/>
```

**Available Libraries:**
- `react-virtuoso` - For chat messages and logs
- `react-window` - For grid layouts

---

## 🧪 Testing

### 17. Testing Requirements
**Rule:** Write tests for critical paths and complex logic.

**Test Priority (High to Low):**
1. API client functions (`lib/api.ts`)
2. Custom hooks with complex logic
3. Utility functions with business logic
4. Critical UI workflows (task creation, execution)

**DO:**
```typescript
// lib/__tests__/api.test.ts
import { describe, it, expect, vi } from 'vitest';
import { tasksApi } from '../api';

describe('tasksApi', () => {
  it('should create a task', async () => {
    // Test implementation
  });
});
```

**Test Commands:**
```bash
npm run test          # Watch mode
npm run test:run      # Single run
npm run test:ui       # UI mode
npm run test:coverage # With coverage
```

### 18. Test File Location
**Rule:** Colocate tests with the code they test.

```
lib/
├── api.ts
└── __tests__/
    └── api.test.ts

hooks/
├── useTaskExecution.ts
└── __tests__/
    └── useTaskExecution.test.ts
```

---

## 🎨 UI & Styling

### 19. Component Library
**Rule:** Use shadcn/ui components for all UI primitives.

**DO:**
```typescript
import { Button } from '@/components/ui/button';
import { Card, CardHeader, CardContent } from '@/components/ui/card';

<Card>
  <CardHeader>Task Details</CardHeader>
  <CardContent>
    <Button onClick={handleEdit}>Edit</Button>
  </CardContent>
</Card>
```

**Available shadcn/ui components:** (32 components in `components/ui/shadcn-io/`)
- Layout: Card, Separator, ScrollArea, Tabs
- Forms: Button, Input, Select, Checkbox, Switch
- Feedback: Alert, Toast, Dialog, Loader
- Navigation: Dropdown, Tooltip

### 20. Tailwind CSS Guidelines
**Rule:** Use Tailwind utility classes. Avoid inline styles.

**DO:**
```typescript
<div className="flex items-center gap-4 p-4 border rounded-lg bg-background">
  <span className="text-sm text-muted-foreground">Status:</span>
  <Badge variant="success">Completed</Badge>
</div>
```

**DON'T:**
```typescript
// ❌ Inline styles
<div style={{ display: 'flex', gap: '16px', padding: '16px' }}>
  <span style={{ fontSize: '14px', color: '#666' }}>Status:</span>
</div>
```

**Exception:** Dynamic styles based on data (use CSS variables)
```typescript
<div style={{ color: `hsl(var(--${statusColor}))` }}>
```

### 21. Responsive Design
**Rule:** Mobile-first approach with Tailwind breakpoints.

**Breakpoints:**
- `sm:` - 640px
- `md:` - 768px
- `lg:` - 1024px
- `xl:` - 1280px

**DO:**
```typescript
<div className="flex flex-col md:flex-row gap-4">
  <aside className="w-full md:w-64">Sidebar</aside>
  <main className="flex-1">Content</main>
</div>
```

---

## 🌐 Internationalization (i18n)

### 22. i18n Usage
**Rule:** All user-facing text MUST be internationalized.

**DO:**
```typescript
import { useTranslation } from 'react-i18next';

function TaskCard({ task }: { task: Task }) {
  const { t } = useTranslation('tasks');

  return (
    <Card>
      <h3>{t('tasks:title')}</h3>
      <p>{t('tasks:status', { status: task.status })}</p>
    </Card>
  );
}
```

**Available Namespaces:**
- `common` - Shared text (buttons, states, etc.)
- `tasks` - Task-related text
- `projects` - Project-related text
- `settings` - Settings text

**DON'T:**
```typescript
// ❌ Hardcoded English text
<Button>Create Task</Button>
<p>Task completed successfully</p>
```

---

## 🚀 Build & Bundle Optimization

### 23. Import Optimization
**Rule:** Use path aliases and avoid deep imports.

**DO:**
```typescript
import { Button } from '@/components/ui/button';
import { tasksApi } from '@/lib/api';
import { Task } from 'shared/types';
```

**DON'T:**
```typescript
// ❌ Relative paths
import { Button } from '../../../components/ui/button';
// ❌ Deep library imports (increases bundle size)
import map from 'lodash/map';
import filter from 'lodash/filter';
```

**Exception:** Tree-shakeable libraries like `lodash-es`:
```typescript
// ✅ OK if using lodash-es
import { map, filter } from 'lodash-es';
```

### 24. Dynamic Imports
**Rule:** Use dynamic imports for heavy dependencies used conditionally.

**DO:**
```typescript
// Only load heavy library when needed
const handleExport = async () => {
  const { saveAs } = await import('file-saver');
  saveAs(blob, 'export.json');
};
```

---

## 📝 Code Quality

### 25. ESLint & Prettier
**Rule:** All code MUST pass linting and formatting checks.

**Before committing:**
```bash
npm run check        # TypeScript check
npm run lint         # ESLint (max 50 warnings)
npm run format:check # Prettier check
```

**Auto-fix:**
```bash
npm run lint:fix     # Fix linting issues
npm run format       # Format code
```

### 26. Naming Conventions

**Files:**
- Components: `PascalCase.tsx` (e.g., `TaskCard.tsx`)
- Hooks: `camelCase.ts` (e.g., `useTaskExecution.ts`)
- Utils: `kebab-case.ts` (e.g., `status-labels.ts`)
- Types: Match the source (e.g., `shared/types.ts`)

**Variables & Functions:**
- Variables: `camelCase` (e.g., `selectedTask`)
- Functions: `camelCase` (e.g., `handleTaskEdit`)
- Components: `PascalCase` (e.g., `TaskCard`)
- Constants: `UPPER_SNAKE_CASE` (e.g., `MAX_RETRY_COUNT`)

**Booleans:**
- Prefix with `is`, `has`, `should`, `can` (e.g., `isLoading`, `hasError`)

### 27. Comments & Documentation
**Rule:** Write self-documenting code. Use comments for "why", not "what".

**DO:**
```typescript
// Workaround for Sentry ErrorBoundary type mismatch with Vite's import.meta.env
const errorObj = error instanceof Error ? error : new Error(String(error));

// Using 'any' here because RJSF library requires it for dynamic form data
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ExecutorConfig = any;
```

**DON'T:**
```typescript
// ❌ Obvious comment
// Set loading to true
setLoading(true);

// ❌ Outdated comment
// TODO: Fix this later (written 6 months ago)
```

---

## 🔒 Security

### 28. XSS Prevention
**Rule:** Never use `dangerouslySetInnerHTML` without sanitization.

**DO:**
```typescript
import DOMPurify from 'dompurify';

const sanitizedHtml = DOMPurify.sanitize(userInput);
<div dangerouslySetInnerHTML={{ __html: sanitizedHtml }} />
```

**Better: Use a markdown renderer**
```typescript
import Markdown from 'markdown-to-jsx';

<Markdown>{userInput}</Markdown>
```

### 29. Sensitive Data
**Rule:** Never log or expose sensitive data.

**DON'T:**
```typescript
// ❌ Logging tokens/keys
console.log('Auth token:', authToken);
console.log('API key:', apiKey);
```

---

## 🗂️ File Organization

### 30. Maximum File Sizes
**Rule:** Keep files focused and maintainable.

**Limits:**
- Component files: Max 300 lines
- Hook files: Max 200 lines
- Utility files: Max 150 lines
- API modules: Max 1000 lines (exception: `lib/api.ts` is centralized)

**If exceeded:** Split into multiple files or extract logic into hooks/utils.

### 31. Index Files
**Rule:** Use index files for cleaner imports, but don't overuse them.

**DO:**
```typescript
// components/tasks/index.ts
export { TaskCard } from './TaskCard';
export { TaskList } from './TaskList';
export { TaskDetails } from './TaskDetails';

// Import
import { TaskCard, TaskList } from '@/components/tasks';
```

**DON'T:**
```typescript
// ❌ Re-exporting everything creates circular dependencies
export * from './TaskCard';
export * from './TaskList';
```

---

## 🔄 Git Workflow

### 32. Commit Messages
**Rule:** Follow conventional commits format.

**Format:**
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `docs:` - Documentation
- `style:` - Formatting, missing semicolons, etc.
- `perf:` - Performance improvement
- `chore:` - Maintenance tasks

**Examples:**
```
feat(tasks): add task filtering by status
fix(api): handle network timeout errors
refactor(ui): consolidate button variants
test(api): add tests for tasksApi client
```

### 33. Branch Naming
**Rule:** Use descriptive branch names with prefixes.

**Format:**
```
<type>/<short-description>
```

**Examples:**
```
feat/task-kanban-view
fix/diff-rendering-bug
refactor/state-management-zustand
test/api-client-coverage
```

---

## 🎓 Learning Resources

### Key Files to Study
1. `lib/api.ts` - API client patterns
2. `hooks/useConversationHistory.ts` - SSE and real-time updates
3. `components/tasks/TaskKanbanBoard.tsx` - Complex component patterns
4. `stores/useUIStore.ts` - Zustand state management
5. `App.tsx` - Routing and code splitting

### External Documentation
- [React 18 Docs](https://react.dev/)
- [TanStack Query](https://tanstack.com/query/latest)
- [Zustand](https://docs.pmnd.rs/zustand/getting-started/introduction)
- [Tailwind CSS](https://tailwindcss.com/docs)
- [shadcn/ui](https://ui.shadcn.com/)
- [Vite](https://vitejs.dev/)

---

## ✅ Pre-Commit Checklist

Before committing frontend changes, verify:

- [ ] `npm run check` passes (TypeScript)
- [ ] `npm run lint` passes (max 50 warnings)
- [ ] `npm run format:check` passes
- [ ] `npm run test:run` passes (if tests exist)
- [ ] No `console.log` statements (use proper logging)
- [ ] All user-facing text is internationalized
- [ ] No hardcoded API URLs
- [ ] Error boundaries added for new routes
- [ ] New heavy dependencies are lazy-loaded

---

## 🆘 When in Doubt

1. **Check existing patterns** - Look for similar code in the codebase
2. **Follow TypeScript errors** - They usually point to the right solution
3. **Keep it simple** - Don't over-engineer
4. **Ask questions** - Better to clarify than make wrong assumptions

---

## 📊 Architecture Metrics

**Current State (After Improvements):**
- Total TypeScript files: 238
- Total lines: ~32,512
- UI Components: 32 (shadcn/ui)
- Custom Hooks: 37
- Contexts: 10 (reduced from 12)
- Zustand Stores: 4
- Test Coverage: In progress
- Bundle Size: ~2.4 MB (main chunks)
- Code Splitting: ✅ Enabled (8+ chunks)
- Error Boundaries: ✅ Page-level
- TypeScript Strict Mode: ✅ Enabled
- ESLint Max Warnings: 50 (down from 110)

---

**Last Updated:** 2025-11-12
**Version:** 1.0.0
