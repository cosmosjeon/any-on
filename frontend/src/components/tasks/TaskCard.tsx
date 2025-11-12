import { useCallback, useState } from 'react';
import { CheckCircle, Link, Loader2, XCircle } from 'lucide-react';
import type { TaskWithAttemptStatus } from 'shared/types';
import { ActionsDropdown } from '@/components/ui/ActionsDropdown';
import { Button } from '@/components/ui/button';
import { useNavigateWithSearch } from '@/hooks';
import { paths } from '@/lib/paths';
import { attemptsApi } from '@/lib/api';

type Task = TaskWithAttemptStatus;

interface TaskCardProps {
  task: Task;
  projectId: string;
  onViewDetails: (task: Task) => void;
}

export function TaskCard({ task, projectId, onViewDetails }: TaskCardProps) {
  const navigate = useNavigateWithSearch();
  const [isNavigatingToParent, setIsNavigatingToParent] = useState(false);

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      // Don't trigger if clicking on buttons or interactive elements
      const target = e.target as HTMLElement;
      if (
        target.closest('button') ||
        target.closest('[role="menuitem"]') ||
        target.closest('[data-radix-collection-item]')
      ) {
        return;
      }
      onViewDetails(task);
    },
    [task, onViewDetails]
  );

  const handleParentClick = useCallback(
    async (e: React.MouseEvent) => {
      e.stopPropagation();
      if (!task.parent_task_attempt || isNavigatingToParent) return;

      setIsNavigatingToParent(true);
      try {
        const parentAttempt = await attemptsApi.get(task.parent_task_attempt);
        navigate(
          paths.attempt(
            projectId,
            parentAttempt.task_id,
            task.parent_task_attempt
          )
        );
      } catch (error) {
        console.error('Failed to navigate to parent task attempt:', error);
        setIsNavigatingToParent(false);
      }
    },
    [task.parent_task_attempt, projectId, navigate, isNavigatingToParent]
  );

  return (
    <div onClick={handleClick} className="cursor-pointer">
      <div className="flex flex-1 gap-2 items-center min-w-0">
        <h4 className="flex-1 min-w-0 line-clamp-2 font-medium text-sm">
          {task.title}
        </h4>
        <div className="flex items-center gap-1 flex-shrink-0">
          {/* In Progress Spinner */}
          {task.has_in_progress_attempt && (
            <Loader2 className="h-4 w-4 animate-spin text-blue-500" />
          )}
          {/* Merged Indicator */}
          {task.has_merged_attempt && (
            <CheckCircle className="h-4 w-4 text-green-500" />
          )}
          {/* Failed Indicator */}
          {task.last_attempt_failed && !task.has_merged_attempt && (
            <XCircle className="h-4 w-4 text-destructive" />
          )}
          {/* Parent Task Indicator */}
          {task.parent_task_attempt && (
            <Button
              variant="icon"
              onClick={handleParentClick}
              onPointerDown={(e) => e.stopPropagation()}
              onMouseDown={(e) => e.stopPropagation()}
              disabled={isNavigatingToParent}
              title="Navigate to parent task attempt"
            >
              <Link className="h-4 w-4" />
            </Button>
          )}
          {/* Actions Menu */}
          <ActionsDropdown task={task} />
        </div>
      </div>
      {task.description && (
        <p className="text-xs text-muted-foreground break-words mt-2">
          {task.description.length > 100
            ? `${task.description.substring(0, 100)}...`
            : task.description}
        </p>
      )}
    </div>
  );
}
