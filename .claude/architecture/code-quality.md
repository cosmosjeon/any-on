# Code Quality Standards

**Guidelines for maintaining high-quality, maintainable code**

---

## TDD (Test-Driven Development)

### Core Cycle: Red → Green → Refactor

1. **RED Phase**: Write failing test FIRST for simplest scenario
2. **GREEN Phase**: Write MINIMAL code to pass (no premature optimization)
3. **REFACTOR Phase**: Remove duplication, improve naming, keep tests passing

### Test Quality: FIRST Principles

- **Fast**: Milliseconds, not seconds
- **Independent**: No shared state between tests
- **Repeatable**: Same result every time
- **Self-validating**: Pass/fail, no manual checks
- **Timely**: Written just before code

### Test Structure: AAA Pattern

```typescript
describe('TaskService', () => {
  it('should create a task with valid data', () => {
    // Arrange: Set up test data and dependencies
    const taskData = { title: 'Test Task', projectId: '123' };

    // Act: Execute the function/method
    const result = taskService.create(taskData);

    // Assert: Verify expected outcome
    expect(result).toHaveProperty('id');
    expect(result.title).toBe('Test Task');
  });
});
```

### Implementation Flow

1. List scenarios before coding
2. Pick one scenario → Write test
3. Run test → See it fail (Red)
4. Implement → Make it pass (Green)
5. Refactor → Clean up (Still Green)
6. Commit → Small, frequent commits
7. Repeat → Next scenario

---

## Clean Code Principles

### Core Principles

- **DRY** (Don't Repeat Yourself) - Eliminate duplication ruthlessly
- **KISS** (Keep It Simple, Stupid) - Simplest solution that works
- **YAGNI** (You Aren't Gonna Need It) - Build only what's needed now
- **SOLID** - Apply all five principles consistently
- **Boy Scout Rule** - Leave code cleaner than you found it

### Naming Conventions

**Intention-Revealing Names**
```typescript
// ✅ Good
const isTaskCompleted = task.status === 'completed';
const MAX_RETRY_ATTEMPTS = 3;
const getUserById = (id: string) => { ... };

// ❌ Bad
const flag = task.status === 'completed';  // What flag?
const MAX = 3;  // Max what?
const get = (id: string) => { ... };  // Get what?
```

**Conventions:**
- Classes: **Nouns** (PascalCase) - `TaskManager`, `UserService`
- Functions: **Verbs** (camelCase) - `createTask`, `fetchUser`
- Booleans: **is/has/can/should** prefix - `isLoading`, `hasError`, `canEdit`
- Constants: **UPPER_SNAKE_CASE** - `MAX_RETRY_COUNT`, `API_BASE_URL`
- No magic numbers - use named constants

**Avoid Abbreviations**
```typescript
// ✅ Good
const userRepository = new UserRepository();
const authenticateUser = (credentials) => { ... };

// ❌ Bad
const usrRepo = new UserRepository();
const authUsr = (creds) => { ... };
```

---

## Functions & Methods

### Single Responsibility Principle
Each function should do ONE thing and do it well.

```typescript
// ✅ Good - Each function has one responsibility
function validateTask(task: Task): ValidationResult {
  return {
    isValid: task.title.length > 0 && task.projectId != null,
    errors: getValidationErrors(task),
  };
}

function saveTask(task: Task): Promise<Task> {
  return tasksApi.create(task);
}

function createAndSaveTask(taskData: CreateTask): Promise<Task> {
  const validation = validateTask(taskData);
  if (!validation.isValid) {
    throw new ValidationError(validation.errors);
  }
  return saveTask(taskData);
}

// ❌ Bad - Doing too many things
function processTask(task: Task) {
  // Validation
  if (task.title.length === 0) throw new Error('Invalid');
  // Save
  const result = await tasksApi.create(task);
  // Log
  console.log('Task created:', result.id);
  // Update UI
  showNotification('Task created!');
  // Return
  return result;
}
```

### Size Limits
- **Maximum 20 lines** per function (prefer under 10)
- **Maximum 3 parameters** (use objects for more)
- If longer, extract helper functions

### No Side Effects
```typescript
// ✅ Good - Pure function
function calculateTotal(items: Item[]): number {
  return items.reduce((sum, item) => sum + item.price, 0);
}

// ❌ Bad - Side effect (mutation)
function calculateTotal(items: Item[]): number {
  items.forEach(item => item.calculated = true);  // Side effect!
  return items.reduce((sum, item) => sum + item.price, 0);
}
```

### Early Returns
Prefer early returns over nested conditions.

```typescript
// ✅ Good - Early returns
function processTask(task: Task | null): string {
  if (task == null) return 'No task';
  if (task.status === 'completed') return 'Already completed';
  if (task.assignee == null) return 'Not assigned';

  return executeTask(task);
}

// ❌ Bad - Nested conditions
function processTask(task: Task | null): string {
  if (task != null) {
    if (task.status !== 'completed') {
      if (task.assignee != null) {
        return executeTask(task);
      } else {
        return 'Not assigned';
      }
    } else {
      return 'Already completed';
    }
  } else {
    return 'No task';
  }
}
```

---

## Code Structure

### Cyclomatic Complexity
Keep complexity < 10 per function.

**Complexity = number of decision points + 1**
- Each `if`, `else if`, `case`, `&&`, `||`, `?:` adds +1

```typescript
// ✅ Good - Complexity: 3
function getTaskStatus(task: Task): string {
  if (task.completedAt != null) return 'completed';
  if (task.startedAt != null) return 'in_progress';
  return 'pending';
}

// ❌ Bad - Complexity: 8
function validateAndProcessTask(task: Task) {
  if (task != null && task.title != null && task.title.length > 0) {
    if (task.projectId != null && task.assignee != null) {
      if (task.priority === 'high' || task.dueDate < Date.now()) {
        // ... complex logic
      }
    }
  }
}
```

### Maximum Nesting Depth: 3 Levels
```typescript
// ✅ Good - 2 levels
function processTasks(tasks: Task[]) {
  for (const task of tasks) {
    if (task.status === 'pending') {
      executeTask(task);
    }
  }
}

// ❌ Bad - 5 levels
function processTasks(tasks: Task[]) {
  if (tasks != null) {
    for (const task of tasks) {
      if (task.status === 'pending') {
        if (task.assignee != null) {
          if (task.priority === 'high') {
            // Too deep!
          }
        }
      }
    }
  }
}
```

### Organization by Feature
```
# ✅ Good - Feature-based
features/
├── tasks/
│   ├── TaskList.tsx
│   ├── TaskCard.tsx
│   ├── useTaskService.ts
│   └── tasksApi.ts
└── projects/
    ├── ProjectList.tsx
    ├── ProjectCard.tsx
    └── projectsApi.ts

# ❌ Bad - Type-based
src/
├── components/
│   ├── TaskList.tsx
│   └── ProjectList.tsx
├── hooks/
│   └── useTaskService.ts
└── api/
    ├── tasksApi.ts
    └── projectsApi.ts
```

---

## Error Handling

### Fail Fast with Clear Messages
```typescript
// ✅ Good
function divideNumbers(a: number, b: number): number {
  if (b === 0) {
    throw new Error(`Cannot divide ${a} by zero`);
  }
  return a / b;
}

// ❌ Bad
function divideNumbers(a: number, b: number): number {
  return b === 0 ? 0 : a / b;  // Silent failure!
}
```

### Use Exceptions Over Error Codes
```typescript
// ✅ Good
async function createTask(data: CreateTask): Promise<Task> {
  if (!data.title) {
    throw new ValidationError('Task title is required');
  }

  try {
    return await tasksApi.create(data);
  } catch (error) {
    throw new ApiError('Failed to create task', { cause: error });
  }
}

// ❌ Bad
async function createTask(data: CreateTask): Promise<Task | null> {
  if (!data.title) return null;  // What went wrong?

  const result = await tasksApi.create(data);
  return result ? result : null;  // Silent failure
}
```

### Handle Errors at Appropriate Levels
```typescript
// ✅ Good - Handle at API boundary
export const tasksApi = {
  async create(data: CreateTask): Promise<Result<Task, ApiError>> {
    try {
      const response = await fetch('/api/tasks', {
        method: 'POST',
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        return Err(new ApiError(`HTTP ${response.status}`, response));
      }

      const task = await response.json();
      return Ok(task);
    } catch (error) {
      return Err(new ApiError('Network error', error));
    }
  },
};

// Component handles result
const result = await tasksApi.create(taskData);
if (result.ok) {
  showSuccess('Task created');
} else {
  showError(result.error.message);
}
```

### Never Catch Generic Exceptions
```typescript
// ✅ Good - Specific error handling
try {
  await saveTask(task);
} catch (error) {
  if (error instanceof ValidationError) {
    showValidationErrors(error.fields);
  } else if (error instanceof NetworkError) {
    showRetryDialog();
  } else {
    throw error;  // Re-throw unknown errors
  }
}

// ❌ Bad - Catch everything
try {
  await saveTask(task);
} catch (error) {
  console.log('Something went wrong');  // What? Where? Why?
}
```

### Log Errors with Context
```typescript
// ✅ Good
try {
  await tasksApi.update(taskId, updates);
} catch (error) {
  console.error('Failed to update task', {
    taskId,
    updates,
    error,
    timestamp: new Date().toISOString(),
  });
  throw error;
}

// ❌ Bad
try {
  await tasksApi.update(taskId, updates);
} catch (error) {
  console.log(error);  // No context!
}
```

---

## TypeScript/React Best Practices

### Avoid Code Duplication
```typescript
// ✅ Good - Extract shared logic
function useTaskActions(taskId: string) {
  const queryClient = useQueryClient();

  const invalidateTasks = useCallback(() => {
    queryClient.invalidateQueries(['tasks']);
    queryClient.invalidateQueries(['task', taskId]);
  }, [queryClient, taskId]);

  const updateTask = useMutation({
    mutationFn: (data) => tasksApi.update(taskId, data),
    onSuccess: invalidateTasks,
  });

  const deleteTask = useMutation({
    mutationFn: () => tasksApi.delete(taskId),
    onSuccess: invalidateTasks,
  });

  return { updateTask, deleteTask };
}
```

### Explicit Boolean Conditions
```typescript
// ✅ Good - Explicit booleans
{isLoading && <Spinner />}
{items.length > 0 && <ItemList items={items} />}
{user?.name != null && <UserName name={user.name} />}

// ❌ Bad - Ambiguous truthiness
{data && <DataComponent />}  // What if data = 0?
{count && <Counter />}  // What if count = 0?
{text && <Text />}  // What if text = ""?
```

### Nullish Comparisons
```typescript
// ✅ Good - Explicit null checks
const value = userInput != null ? userInput : defaultValue;
const name = user?.name != null ? user.name : 'Anonymous';

// ❌ Bad - Implicit fallbacks
const value = userInput || defaultValue;  // Fails for 0, '', false
```

### Avoid Nested Ternaries
```typescript
// ✅ Good - Clear logic
function getStatusColor(status: TaskStatus): string {
  if (status === 'completed') return 'green';
  if (status === 'in_progress') return 'blue';
  if (status === 'failed') return 'red';
  return 'gray';
}

// ❌ Bad - Nested ternaries
const color = status === 'completed' ? 'green'
  : status === 'in_progress' ? 'blue'
  : status === 'failed' ? 'red'
  : 'gray';
```

---

## Pre-Commit Checklist

- [ ] All tests pass
- [ ] No linting errors
- [ ] No `console.log` statements (use proper logging)
- [ ] No commented-out code
- [ ] No TODOs without issue tickets
- [ ] Performance is acceptable
- [ ] Security has been considered
- [ ] Documentation is updated
- [ ] Function complexity < 10
- [ ] Max nesting depth: 3
- [ ] Functions < 20 lines
- [ ] Clear, intention-revealing names

---

For language-specific guidelines, see:
- Frontend: `.claude/rules/02-frontend.md`
- Backend: `.claude/rules/03-backend.md` (TBD)
