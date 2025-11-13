import { memo, useCallback, useEffect, useMemo, useRef } from 'react';
import { EpicCard } from './EpicCard';
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

import { extendedStatusLabels, extendedStatusColors } from '@/utils/status-labels';

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

// Extended status type including frontend-only statuses
type ExtendedStatus = TaskStatus | 'plan';

function TaskKanbanBoard({
  groupedTasks,
  onDragEnd,
  onViewTaskDetails,
  selectedTask,
  onCreateTask,
  projectId,
}: TaskKanbanBoardProps) {
  const cardRefs = useRef<Record<string, HTMLDivElement | null>>({});

  const { columns, data, epicStoriesMap } = useMemo(() => {
    // Define column order with Plan inserted after Backlog
    const columnOrder: ExtendedStatus[] = ['todo', 'plan', 'inprogress', 'inreview', 'done', 'cancelled'];

    const columns: KanbanColumn[] = columnOrder.map((status) => ({
      id: status,
      name: extendedStatusLabels[status] || status,
      color: extendedStatusColors[status] || 'hsl(var(--muted-foreground))',
    }));

    // Separate backlog tasks into Epics and Stories based on title prefix
    const backlogTasks = groupedTasks['todo'] || [];
    const epics = backlogTasks.filter(task => task.title.toLowerCase().startsWith('epic:'));
    const stories = backlogTasks.filter(task => task.title.toLowerCase().startsWith('story:'));
    const otherTasks = backlogTasks.filter(task => 
      !task.title.toLowerCase().startsWith('epic:') && 
      !task.title.toLowerCase().startsWith('story:')
    );

    // Map stories to their parent task attempt (for Epic-Story relationship)
    const epicStoriesMap = new Map<string, Task[]>();
    
    // For prototype, show all stories under each epic
    epics.forEach(epic => {
      epicStoriesMap.set(epic.id, stories);
    });

    // Create data array - for backlog, include Epics and other tasks (not Stories)
    const data: KanbanTaskItem[] = [
      // Backlog: Epics and other tasks (Stories will be shown inside Epic cards)
      ...epics.map(task => ({
        id: task.id,
        name: task.title,
        column: 'todo',
        task,
      })),
      ...otherTasks.map(task => ({
        id: task.id,
        name: task.title,
        column: 'todo',
        task,
      })),
      // Other columns: all tasks
      ...Object.entries(groupedTasks)
        .filter(([status]) => status !== 'todo')
        .flatMap(([status, tasks]) =>
          tasks.map((task) => ({
            id: task.id,
            name: task.title,
            column: status,
            task,
          }))
        ),
    ];

    return { columns, data, epicStoriesMap };
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
        <KanbanBoard key={column.id} id={column.id} className="bg-muted/30">
          <KanbanHeader className="flex items-center justify-between px-2 py-1.5 bg-background/80 backdrop-blur-sm border-b">
            <div className="flex items-center gap-1.5">
              <div
                className="h-1.5 w-1.5 rounded-full"
                style={{ backgroundColor: column.color }}
              />
              <span className="font-bold text-[10px] tracking-tight">
                {column.name}
              </span>
              <div className="flex items-center justify-center min-w-[1.25rem] h-3.5 px-1 rounded-full bg-muted text-[8px] font-semibold text-muted-foreground">
                {data.filter((item) => item.column === column.id).length}
              </div>
            </div>
            {onCreateTask && (
              <button
                onClick={onCreateTask}
                className="flex items-center justify-center h-4 w-4 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors text-xs"
                aria-label="Add task"
              >
                +
              </button>
            )}
          </KanbanHeader>
          <KanbanCards id={column.id} className="p-1.5 gap-1.5">
            {(item) => {
              const taskItem = item as KanbanTaskItem;
              const isSelected = selectedTask?.id === taskItem.id;
              const isBacklog = column.id === 'todo';
              const isEpic = taskItem.task.title.toLowerCase().startsWith('epic:');
              const stories = isBacklog && isEpic ? (epicStoriesMap.get(taskItem.id) || []) : [];

              return (
                <div
                  key={taskItem.id}
                  ref={(el) => {
                    cardRefs.current[taskItem.id] = el;
                  }}
                >
                  {isBacklog && isEpic ? (
                    <EpicCard
                      epic={taskItem.task}
                      stories={stories}
                      projectId={projectId}
                      onViewDetails={handleCardClick}
                    />
                  ) : (
                    <KanbanCard
                      id={taskItem.id}
                      name={taskItem.name}
                      column={taskItem.column}
                      className={cn(
                        'border-border/40 bg-card hover:border-border hover:shadow-lg transition-all duration-200',
                        isSelected && 'ring-1 ring-primary'
                      )}
                    >
                      <TaskCard
                        task={taskItem.task}
                        projectId={projectId}
                        onViewDetails={handleCardClick}
                      />
                    </KanbanCard>
                  )}
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
