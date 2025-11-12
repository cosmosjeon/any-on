# Frontend-First Development Workflow

**Guide for developing frontend before backend implementation**

---

## When to Use This Approach

✅ **Good for:**
- Prototyping new features quickly
- UI/UX experimentation
- Client demos without backend ready
- Parallel development (separate teams)
- Design validation before backend investment

❌ **Not recommended for:**
- Complex business logic that needs backend validation
- Performance-critical features
- Security-sensitive operations

---

## Step-by-Step Workflow

### Phase 1: API Contract Definition

**Before writing ANY code, document the API contract:**

```typescript
// docs/api-specs/tasks-api.md or create a TypeScript interface file

/**
 * GET /api/projects/:projectId/tasks
 * Returns all tasks for a project
 */
interface GetTasksRequest {
  projectId: string;
}

interface GetTasksResponse {
  tasks: Task[];
}

/**
 * POST /api/projects/:projectId/tasks
 * Creates a new task
 */
interface CreateTaskRequest {
  projectId: string;
  title: string;
  description?: string;
  priority: 'low' | 'medium' | 'high';
}

interface CreateTaskResponse {
  task: Task;
}

/**
 * Error Responses
 */
interface ApiError {
  error: string;
  message: string;
  statusCode: number;
}
```

**Share this with the backend team** for agreement before proceeding.

---

### Phase 2: Mock API Layer

Create a mock API layer that matches the contract:

```typescript
// frontend/src/lib/api-mock.ts

import { Task, CreateTask } from 'shared/types';

// Mock data store
const mockTasks: Task[] = [
  {
    id: '1',
    project_id: 'proj-1',
    title: 'Design login page',
    description: 'Create wireframes and mockups',
    status: 'in_progress',
    priority: 'high',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  // ... more mock data
];

// Mock API that mimics real API
export const mockTasksApi = {
  getAll: async (projectId: string): Promise<Task[]> => {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 500));

    return mockTasks.filter(t => t.project_id === projectId);
  },

  getById: async (taskId: string): Promise<Task> => {
    await new Promise(resolve => setTimeout(resolve, 300));

    const task = mockTasks.find(t => t.id === taskId);
    if (!task) {
      throw new Error(`Task ${taskId} not found`);
    }
    return task;
  },

  create: async (data: CreateTask): Promise<Task> => {
    await new Promise(resolve => setTimeout(resolve, 400));

    const newTask: Task = {
      id: `task-${Date.now()}`,
      ...data,
      status: 'pending',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    mockTasks.push(newTask);
    return newTask;
  },

  update: async (taskId: string, updates: Partial<Task>): Promise<Task> => {
    await new Promise(resolve => setTimeout(resolve, 400));

    const index = mockTasks.findIndex(t => t.id === taskId);
    if (index === -1) {
      throw new Error(`Task ${taskId} not found`);
    }

    mockTasks[index] = {
      ...mockTasks[index],
      ...updates,
      updated_at: new Date().toISOString(),
    };

    return mockTasks[index];
  },

  delete: async (taskId: string): Promise<void> => {
    await new Promise(resolve => setTimeout(resolve, 300));

    const index = mockTasks.findIndex(t => t.id === taskId);
    if (index === -1) {
      throw new Error(`Task ${taskId} not found`);
    }

    mockTasks.splice(index, 1);
  },
};
```

---

### Phase 3: API Switch Layer

Create a feature flag to switch between mock and real API:

```typescript
// frontend/src/lib/api.ts

import { mockTasksApi } from './api-mock';
import { realTasksApi } from './api-real'; // Will implement later

// Feature flag - set via environment variable or config
const USE_MOCK_API = import.meta.env.VITE_USE_MOCK_API === 'true';

// Export unified API
export const tasksApi = USE_MOCK_API ? mockTasksApi : realTasksApi;

// Or more sophisticated approach with runtime switching:
export function createTasksApi(useMock: boolean = USE_MOCK_API) {
  return useMock ? mockTasksApi : realTasksApi;
}
```

**Environment Setup:**

```bash
# .env.development
VITE_USE_MOCK_API=true

# .env.production
VITE_USE_MOCK_API=false
```

---

### Phase 4: Implement Frontend

Now develop the entire frontend using the mock API:

```typescript
// components/TaskList.tsx
import { useQuery } from '@tanstack/react-query';
import { tasksApi } from '@/lib/api';

export function TaskList({ projectId }: { projectId: string }) {
  const { data: tasks, isLoading } = useQuery({
    queryKey: ['tasks', projectId],
    queryFn: () => tasksApi.getAll(projectId),
  });

  if (isLoading) return <Loader />;

  return (
    <div>
      {tasks?.map(task => (
        <TaskCard key={task.id} task={task} />
      ))}
    </div>
  );
}
```

**Benefits at this stage:**
- ✅ Fully functional UI/UX
- ✅ All user flows working
- ✅ Can demo to stakeholders
- ✅ Design iteration without backend dependency

---

### Phase 5: Implement Backend

When ready, implement the Rust backend following the agreed API spec:

```rust
// crates/server/src/routes/tasks.rs

use axum::{
    extract::{Path, State},
    Json,
};
use crate::AppState;

// Request/Response types with TS export
#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
}

pub async fn get_tasks(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<Task>>> {
    let tasks = task_service::get_all(&state.db, &project_id).await?;
    Ok(Json(tasks))
}

pub async fn create_task(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<Task>> {
    let task = task_service::create(&state.db, &project_id, req).await?;
    Ok(Json(task))
}
```

---

### Phase 6: Type Synchronization

After backend implementation, sync types:

```bash
# 1. Generate TypeScript types from Rust
npm run generate-types

# 2. Check for type mismatches
npm run check
```

**If there are type mismatches:**
- Either update frontend to match backend types
- Or update backend to match agreed API contract
- Usually backend types should match the original contract

---

### Phase 7: Connect Real API

Implement the real API client:

```typescript
// frontend/src/lib/api-real.ts

import { Task, CreateTask } from 'shared/types';
import { Result, Ok, Err } from './api-types';

export const realTasksApi = {
  async getAll(projectId: string): Promise<Task[]> {
    const response = await fetch(`/api/projects/${projectId}/tasks`, {
      method: 'GET',
      headers: { 'Content-Type': 'application/json' },
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  },

  async getById(taskId: string): Promise<Task> {
    const response = await fetch(`/api/tasks/${taskId}`, {
      method: 'GET',
      headers: { 'Content-Type': 'application/json' },
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  },

  async create(data: CreateTask): Promise<Task> {
    const response = await fetch(`/api/projects/${data.project_id}/tasks`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  },

  // ... other methods
};
```

---

### Phase 8: Switch to Real API

```bash
# Update .env.development
VITE_USE_MOCK_API=false

# Restart dev server
pnpm run dev
```

**Test thoroughly:**
- All CRUD operations
- Error cases
- Loading states
- Edge cases

---

## Best Practices

### 1. Keep Mock API In Sync

When API contract changes during development:

```typescript
// Update both mock and contract simultaneously
// frontend/src/lib/api-mock.ts
export const mockTasksApi = {
  // Updated method signature
  getAll: async (projectId: string, filters?: TaskFilters): Promise<Task[]> => {
    // ...
  },
};
```

### 2. Realistic Mock Data

```typescript
// Create diverse mock data scenarios
const mockTasks: Task[] = [
  // Happy path
  { id: '1', title: 'Normal task', status: 'completed' },

  // Edge cases
  { id: '2', title: 'Very long title that might cause UI issues...', status: 'pending' },
  { id: '3', title: '', status: 'failed' }, // Empty title

  // Special characters
  { id: '4', title: 'Task with <html> & "quotes"', status: 'in_progress' },

  // Different states
  { id: '5', title: 'High priority urgent', priority: 'high', status: 'pending' },
];
```

### 3. Simulate Network Conditions

```typescript
// frontend/src/lib/api-mock.ts

function simulateNetworkDelay(min = 300, max = 1000): Promise<void> {
  const delay = Math.random() * (max - min) + min;
  return new Promise(resolve => setTimeout(resolve, delay));
}

function simulateNetworkError(errorRate = 0.1): void {
  if (Math.random() < errorRate) {
    throw new Error('Network error (simulated)');
  }
}

export const mockTasksApi = {
  async getAll(projectId: string): Promise<Task[]> {
    await simulateNetworkDelay();
    simulateNetworkError(0.05); // 5% error rate

    return mockTasks.filter(t => t.project_id === projectId);
  },
};
```

### 4. Mock Error Scenarios

```typescript
// frontend/src/lib/api-mock.ts

// Add query params to trigger errors for testing
export const mockTasksApi = {
  async getById(taskId: string): Promise<Task> {
    await simulateNetworkDelay();

    // Simulate 404
    if (taskId === 'not-found') {
      throw new ApiError('Task not found', 404);
    }

    // Simulate 500
    if (taskId === 'error') {
      throw new ApiError('Internal server error', 500);
    }

    const task = mockTasks.find(t => t.id === taskId);
    if (!task) {
      throw new ApiError('Task not found', 404);
    }

    return task;
  },
};
```

### 5. Document Mock Limitations

```typescript
// frontend/src/lib/api-mock.ts

/**
 * Mock API for Tasks
 *
 * LIMITATIONS:
 * - No pagination (returns all tasks)
 * - No real search/filtering
 * - No authentication checks
 * - Data resets on page reload
 * - No server-side validation
 *
 * These will be implemented in the real backend.
 */
```

---

## Transitioning Checklist

When switching from mock to real API:

- [ ] All endpoints implemented in backend
- [ ] Types generated: `npm run generate-types`
- [ ] Type check passes: `npm run check`
- [ ] Real API tested manually (Postman/curl)
- [ ] Feature flag switched: `VITE_USE_MOCK_API=false`
- [ ] All user flows tested
- [ ] Error handling works correctly
- [ ] Loading states work as expected
- [ ] Performance is acceptable
- [ ] Mock API kept for future testing/demos

---

## Common Pitfalls to Avoid

### ❌ DON'T: Hardcode Mock Data in Components

```typescript
// ❌ Bad
function TaskList() {
  const tasks = [
    { id: '1', title: 'Hardcoded task' },
    // ...
  ];

  return <div>{tasks.map(...)}</div>;
}
```

```typescript
// ✅ Good - Always go through API layer
function TaskList({ projectId }) {
  const { data: tasks } = useQuery({
    queryKey: ['tasks', projectId],
    queryFn: () => tasksApi.getAll(projectId),
  });

  return <div>{tasks?.map(...)}</div>;
}
```

### ❌ DON'T: Skip Error Handling

```typescript
// ❌ Bad - Only happy path
const { data } = useQuery({
  queryFn: () => tasksApi.getAll(projectId),
});

// ✅ Good - Handle all states
const { data, isLoading, error } = useQuery({
  queryFn: () => tasksApi.getAll(projectId),
});

if (isLoading) return <Loader />;
if (error) return <ErrorMessage error={error} />;
```

### ❌ DON'T: Forget to Sync Types

```typescript
// When backend changes Task interface:
// 1. Run npm run generate-types immediately
// 2. Fix TypeScript errors in frontend
// 3. Update mock data to match new structure
```

---

## Example: Full Feature Implementation

Let's implement a "Task Comments" feature frontend-first:

### 1. API Contract

```typescript
// docs/api-specs/task-comments.ts

interface Comment {
  id: string;
  task_id: string;
  author: string;
  text: string;
  created_at: string;
}

interface GetCommentsResponse {
  comments: Comment[];
}

interface CreateCommentRequest {
  task_id: string;
  text: string;
}
```

### 2. Mock API

```typescript
// frontend/src/lib/api-mock-comments.ts

const mockComments: Comment[] = [
  {
    id: '1',
    task_id: 'task-1',
    author: 'Alice',
    text: 'Looks good to me!',
    created_at: '2025-11-12T10:00:00Z',
  },
];

export const mockCommentsApi = {
  async getAll(taskId: string): Promise<Comment[]> {
    await simulateNetworkDelay();
    return mockComments.filter(c => c.task_id === taskId);
  },

  async create(data: CreateCommentRequest): Promise<Comment> {
    await simulateNetworkDelay();

    const comment: Comment = {
      id: `comment-${Date.now()}`,
      task_id: data.task_id,
      author: 'Current User', // Mock user
      text: data.text,
      created_at: new Date().toISOString(),
    };

    mockComments.push(comment);
    return comment;
  },
};
```

### 3. Component

```typescript
// components/TaskComments.tsx
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { commentsApi } from '@/lib/api';

export function TaskComments({ taskId }: { taskId: string }) {
  const queryClient = useQueryClient();

  const { data: comments } = useQuery({
    queryKey: ['comments', taskId],
    queryFn: () => commentsApi.getAll(taskId),
  });

  const createMutation = useMutation({
    mutationFn: commentsApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries(['comments', taskId]);
    },
  });

  const handleSubmit = (text: string) => {
    createMutation.mutate({ task_id: taskId, text });
  };

  return (
    <div>
      {comments?.map(comment => (
        <CommentCard key={comment.id} comment={comment} />
      ))}
      <CommentForm onSubmit={handleSubmit} />
    </div>
  );
}
```

### 4. Later: Backend Implementation

```rust
// crates/server/src/routes/comments.rs

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Comment {
    pub id: String,
    pub task_id: String,
    pub author: String,
    pub text: String,
    pub created_at: String,
}

pub async fn get_comments(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Vec<Comment>>> {
    let comments = comment_service::get_all(&state.db, &task_id).await?;
    Ok(Json(comments))
}
```

### 5. Switch to Real API

```bash
npm run generate-types
# Update VITE_USE_MOCK_API=false
pnpm run dev
```

---

## Summary

**Frontend-first development is fully supported and encouraged for:**
- Rapid prototyping
- UI/UX validation
- Parallel development
- Early demos

**Key success factors:**
1. ✅ Document API contract first
2. ✅ Create realistic mock API
3. ✅ Use feature flags to switch
4. ✅ Keep mock and contract in sync
5. ✅ Test all states (loading, error, success)
6. ✅ Sync types when backend is ready

This approach allows you to move fast on the frontend while the backend catches up! 🚀
