import { useCallback } from 'react';
import NiceModal from '@ebay/nice-modal-react';
import { Loader } from '@/components/ui/loader';
import { useProject } from '@/contexts/project-context';
import { useProjectTasks } from '@/hooks/useProjectTasks';
import TaskKanbanBoard from '@/components/tasks/TaskKanbanBoard';
import type { DragEndEvent } from '@/components/ui/shadcn-io/kanban';
import type { TaskWithAttemptStatus, TaskStatus } from 'shared/types';
import { tasksApi } from '@/lib/api';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { AlertTriangle } from 'lucide-react';

type Task = TaskWithAttemptStatus;

// Backend task statuses (excluding frontend-only 'plan')
const TASK_STATUSES = [
  'todo',
  'inprogress',
  'inreview',
  'done',
  'cancelled',
] as const;

export function KanbanPage() {
  const { projectId } = useProject();
  const { tasks, isLoading, error } = useProjectTasks(projectId || '');

  const handleDragEnd = useCallback(
    async (event: DragEndEvent) => {
      const { active, over } = event;
      if (!over || !projectId) return;

      const taskId = active.id as string;
      const newStatus = over.id as TaskStatus;

      try {
        await tasksApi.update(taskId, {
          title: null,
          description: null,
          status: newStatus,
          parent_task_attempt: null,
          image_ids: null,
        });
        // WebSocket will automatically update the tasks
      } catch (error) {
        console.error('Failed to update task status:', error);
      }
    },
    [projectId]
  );

  const handleViewTaskDetails = useCallback(
    (task: Task) => {
      if (!projectId) return;
      NiceModal.show('task-detail', { task, projectId });
    },
    [projectId]
  );

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader message="Loading tasks..." size={32} />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <Alert variant="destructive" className="max-w-md">
          <AlertTriangle className="h-4 w-4" />
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>Failed to load tasks: {error}</AlertDescription>
        </Alert>
      </div>
    );
  }

  const groupedTasks = TASK_STATUSES.reduce(
    (acc, status) => {
      acc[status] = tasks?.filter((task) => task.status === status) || [];
      return acc;
    },
    {} as Record<TaskStatus, Task[]>
  );

  if (!projectId) {
    return (
      <div className="flex items-center justify-center h-full">
        <Alert variant="destructive" className="max-w-md">
          <AlertTriangle className="h-4 w-4" />
          <AlertTitle>No Project Selected</AlertTitle>
          <AlertDescription>
            Please select a project to view the Kanban board.
          </AlertDescription>
        </Alert>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-background">
      {/* Header */}
      <div className="border-b bg-background">
        <div className="flex items-center justify-between px-6 py-4">
          <div>
            <h1 className="text-2xl font-semibold">태스크 보드</h1>
            <p className="text-sm text-muted-foreground mt-1">
              프로젝트의 모든 태스크를 칸반 보드로 시각화하고 관리합니다.
            </p>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 min-h-0 overflow-hidden">
        <TaskKanbanBoard
          groupedTasks={groupedTasks}
          onDragEnd={handleDragEnd}
          onViewTaskDetails={handleViewTaskDetails}
          projectId={projectId}
        />
      </div>
    </div>
  );
}
