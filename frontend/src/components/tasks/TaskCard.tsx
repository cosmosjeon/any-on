import { useCallback, useState } from 'react';
import { CheckCircle, Link, Loader2, XCircle } from 'lucide-react';
import type { TaskWithAttemptStatus } from 'shared/types';
import { ActionsDropdown } from '@/components/ui/ActionsDropdown';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useNavigateWithSearch } from '@/hooks';
import { paths } from '@/lib/paths';
import { attemptsApi } from '@/lib/api';
import { cn } from '@/lib/utils';

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
    <div
      onClick={handleClick}
      className="group cursor-pointer space-y-1.5 w-full h-full"
    >
      {/* Header with Title and Actions */}
      <div className="flex items-start gap-1">
        <h4 className="flex-1 min-w-0 font-semibold text-[10px] leading-tight line-clamp-2 group-hover:text-primary transition-colors">
          {task.title}
        </h4>
        <div className="flex items-center flex-shrink-0" onClick={(e) => e.stopPropagation()}>
          {/* Actions Menu */}
          <ActionsDropdown task={task} />
        </div>
      </div>

      {/* Description */}
      {task.description && (
        <p className="text-[9px] text-muted-foreground leading-snug line-clamp-1">
          {task.description}
        </p>
      )}

      {/* Status Indicators */}
      <div className="flex items-center gap-1 flex-wrap">
        {/* In Progress Badge */}
        {task.has_in_progress_attempt && (
          <Badge
            variant="secondary"
            className="gap-0.5 bg-blue-500/10 text-blue-600 hover:bg-blue-500/20 border-blue-200 pointer-events-none h-4 px-1"
          >
            <Loader2 className="h-2 w-2 animate-spin" />
            <span className="text-[8px] font-medium">Progress</span>
          </Badge>
        )}

        {/* Merged Badge */}
        {task.has_merged_attempt && (
          <Badge
            variant="secondary"
            className="gap-0.5 bg-green-500/10 text-green-600 hover:bg-green-500/20 border-green-200 pointer-events-none h-4 px-1"
          >
            <CheckCircle className="h-2 w-2" />
            <span className="text-[8px] font-medium">Merged</span>
          </Badge>
        )}

        {/* Failed Badge */}
        {task.last_attempt_failed && !task.has_merged_attempt && (
          <Badge
            variant="destructive"
            className="gap-0.5 bg-red-500/10 text-red-600 hover:bg-red-500/20 border-red-200 pointer-events-none h-4 px-1"
          >
            <XCircle className="h-2 w-2" />
            <span className="text-[8px] font-medium">Failed</span>
          </Badge>
        )}

        {/* Parent Task Link */}
        {task.parent_task_attempt && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleParentClick}
            onPointerDown={(e) => e.stopPropagation()}
            onMouseDown={(e) => e.stopPropagation()}
            disabled={isNavigatingToParent}
            className={cn(
              'h-4 gap-0.5 px-1 text-[8px]',
              'hover:bg-accent hover:text-accent-foreground'
            )}
            title="Navigate to parent task attempt"
          >
            <Link className="h-2 w-2" />
            <span>Parent</span>
          </Button>
        )}
      </div>
    </div>
  );
}
