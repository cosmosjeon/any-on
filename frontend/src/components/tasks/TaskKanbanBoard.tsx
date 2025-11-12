import { memo, useCallback, useEffect, useMemo, useRef } from 'react';
import {
  type DragEndEvent,
  KanbanBoard,
  KanbanCard,
  KanbanCards,
  KanbanHeader,
  KanbanProvider,
} from '@/components/ui/shadcn-io/kanban';
import { TaskCard } from './TaskCard';
import type { TaskStatus, TaskWithAttemptStatus } from 'shared/types';
import { cn } from '@/lib/utils';

import { statusBoardColors, statusLabels } from '@/utils/status-labels';

type Task = TaskWithAttemptStatus;

interface TaskKanbanBoardProps {
  groupedTasks: Record<TaskStatus, Task[]>;
  onDragEnd: (event: DragEndEvent) => void;
  onViewTaskDetails: (task: Task) => void;
  selectedTask?: Task;
  onCreateTask?: () => void;
  projectId: string;
}

type KanbanTaskItem = {
  id: string;
  name: string;
  column: string;
  task: Task;
};

type KanbanColumn = {
  id: string;
  name: string;
  color: string;
};

function TaskKanbanBoard({
  groupedTasks,
  onDragEnd,
  onViewTaskDetails,
  selectedTask,
  onCreateTask,
  projectId,
}: TaskKanbanBoardProps) {
  const cardRefs = useRef<Record<string, HTMLDivElement | null>>({});

  const { columns, data } = useMemo(() => {
    const columns: KanbanColumn[] = Object.keys(groupedTasks).map((status) => ({
      id: status,
      name: statusLabels[status as TaskStatus],
      color: statusBoardColors[status as TaskStatus],
    }));

    const data: KanbanTaskItem[] = Object.entries(groupedTasks).flatMap(
      ([status, tasks]) =>
        tasks.map((task) => ({
          id: task.id,
          name: task.title,
          column: status,
          task,
        }))
    );

    return { columns, data };
  }, [groupedTasks]);

  const handleDataChange = () => {
    // This is called during drag operations
    // The actual status update happens in onDragEnd
  };

  const handleCardClick = useCallback(
    (task: Task) => {
      onViewTaskDetails(task);
    },
    [onViewTaskDetails]
  );

  // Scroll to selected task
  useEffect(() => {
    if (selectedTask && cardRefs.current[selectedTask.id]) {
      const el = cardRefs.current[selectedTask.id];
      requestAnimationFrame(() => {
        el?.scrollIntoView({
          block: 'center',
          inline: 'nearest',
          behavior: 'smooth',
        });
      });
    }
  }, [selectedTask]);

  return (
    <KanbanProvider
      columns={columns}
      data={data}
      onDataChange={handleDataChange}
      onDragEnd={onDragEnd}
    >
      {(column) => (
        <KanbanBoard key={column.id} id={column.id}>
          <KanbanHeader className="flex items-center justify-between px-3 py-2">
            <div className="flex items-center gap-2">
              <div
                className="h-3 w-3 rounded-full"
                style={{ backgroundColor: column.color }}
              />
              <span className="font-semibold">{column.name}</span>
              <span className="text-xs text-muted-foreground">
                {data.filter((item) => item.column === column.id).length}
              </span>
            </div>
            {onCreateTask && (
              <button
                onClick={onCreateTask}
                className="text-muted-foreground hover:text-foreground text-xl leading-none"
                aria-label="Add task"
              >
                +
              </button>
            )}
          </KanbanHeader>
          <KanbanCards id={column.id} className="p-3">
            {(item) => {
              const taskItem = item as KanbanTaskItem;
              const isSelected = selectedTask?.id === taskItem.id;
              return (
                <div
                  key={taskItem.id}
                  ref={(el) => {
                    cardRefs.current[taskItem.id] = el;
                  }}
                >
                  <KanbanCard
                    id={taskItem.id}
                    name={taskItem.name}
                    column={taskItem.column}
                    className={cn(
                      isSelected && 'ring-2 ring-primary'
                    )}
                  >
                    <TaskCard
                      task={taskItem.task}
                      projectId={projectId}
                      onViewDetails={handleCardClick}
                    />
                  </KanbanCard>
                </div>
              );
            }}
          </KanbanCards>
        </KanbanBoard>
      )}
    </KanbanProvider>
  );
}

export default memo(TaskKanbanBoard);
