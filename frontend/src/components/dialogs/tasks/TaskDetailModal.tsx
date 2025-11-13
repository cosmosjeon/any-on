import {
  Dialog,
  DialogContent,
} from '@/components/ui/dialog';
import NiceModal, { useModal } from '@ebay/nice-modal-react';
import type { TaskWithAttemptStatus } from 'shared/types';
import { useState, useMemo, useCallback } from 'react';
import { useTaskAttempts } from '@/hooks/useTaskAttempts';
import { useTaskAttempt } from '@/hooks/useTaskAttempt';
import { PreviewPanel } from '@/components/panels/PreviewPanel';
import { DiffsPanel } from '@/components/panels/DiffsPanel';
import { ExecutionProcessesProvider } from '@/contexts/ExecutionProcessesContext';
import { ClickedElementsProvider } from '@/contexts/ClickedElementsProvider';
import { ReviewProvider } from '@/contexts/ReviewProvider';
import TaskAttemptPanel from '@/components/panels/TaskAttemptPanel';
import TaskPanel from '@/components/panels/TaskPanel';
import { NewCard, NewCardHeader } from '@/components/ui/new-card';
import TodoPanel from '@/components/tasks/TodoPanel';
import { TaskPanelHeaderActions } from '@/components/panels/TaskPanelHeaderActions';
import { AttemptHeaderActions } from '@/components/panels/AttemptHeaderActions';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb';
import { useBranchStatus } from '@/hooks';
import type { LayoutMode } from '@/components/layout/TasksLayout';

export interface TaskDetailModalProps {
  task: TaskWithAttemptStatus;
  projectId: string;
}

const TaskDetailModal = NiceModal.create<TaskDetailModalProps>((props) => {
  const modal = useModal();
  const { task } = props;
  const [mode, setMode] = useState<LayoutMode>(null);
  const [gitError] = useState<string | null>(null);

  const { data: attempts = [] } = useTaskAttempts(task.id, {
    enabled: !!task.id,
  });

  const latestAttemptId = useMemo(() => {
    if (!attempts?.length) return undefined;
    return [...attempts].sort((a, b) => {
      const diff =
        new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      if (diff !== 0) return diff;
      return a.id.localeCompare(b.id);
    })[0].id;
  }, [attempts]);

  const { data: attempt } = useTaskAttempt(latestAttemptId);
  useBranchStatus(attempt?.id); // Load branch status for children

  const isTaskView = !attempt;

  const handleClose = useCallback(() => {
    modal.hide();
  }, [modal]);

  const truncateTitle = (title: string | undefined, maxLength = 30) => {
    if (!title) return 'Task';
    if (title.length <= maxLength) return title;
    const truncated = title.substring(0, maxLength);
    const lastSpace = truncated.lastIndexOf(' ');
    return lastSpace > 0
      ? `${truncated.substring(0, lastSpace)}...`
      : `${truncated}...`;
  };

  // Right header (same as in project-tasks)
  const rightHeader = (
    <NewCardHeader
      className="shrink-0"
      actions={
        isTaskView ? (
          <TaskPanelHeaderActions
            task={task}
            onClose={handleClose}
          />
        ) : (
          <AttemptHeaderActions
            mode={mode}
            onModeChange={setMode}
            task={task}
            attempt={attempt ?? null}
            onClose={handleClose}
          />
        )
      }
    >
      <div className="mx-auto w-full">
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem>
              {isTaskView ? (
                <BreadcrumbPage>
                  {truncateTitle(task?.title)}
                </BreadcrumbPage>
              ) : (
                <BreadcrumbLink
                  className="cursor-pointer hover:underline"
                  onClick={() => {}}
                >
                  {truncateTitle(task?.title)}
                </BreadcrumbLink>
              )}
            </BreadcrumbItem>
            {!isTaskView && (
              <>
                <BreadcrumbSeparator />
                <BreadcrumbItem>
                  <BreadcrumbPage>
                    {attempt?.branch || 'Task Attempt'}
                  </BreadcrumbPage>
                </BreadcrumbItem>
              </>
            )}
          </BreadcrumbList>
        </Breadcrumb>
      </div>
    </NewCardHeader>
  );

  // Attempt content (same as in project-tasks)
  const attemptContent = (
    <NewCard className="h-full min-h-0 flex flex-col bg-diagonal-lines bg-muted border-0">
      {isTaskView ? (
        <TaskPanel task={task} />
      ) : (
        <TaskAttemptPanel attempt={attempt} task={task}>
          {({ logs, followUp }) => (
            <>
              {gitError && (
                <div className="mx-4 mt-4 p-3 bg-red-50 border border-red-200 rounded">
                  <div className="text-destructive text-sm">{gitError}</div>
                </div>
              )}
              <div className="flex-1 min-h-0 flex flex-col">{logs}</div>

              <div className="shrink-0 border-t">
                <div className="mx-auto w-full max-w-[50rem]">
                  <TodoPanel />
                </div>
              </div>

              <div className="shrink-0 border-t">
                <div className="mx-auto w-full max-w-[50rem]">{followUp}</div>
              </div>
            </>
          )}
        </TaskAttemptPanel>
      )}
    </NewCard>
  );

  // Aux content (Preview/Diffs)
  const auxContent = mode ? (
    <div className="relative h-full w-full">
      {mode === 'preview' && attempt && <PreviewPanel />}
      {mode === 'diffs' && attempt && (
        <DiffsPanel selectedAttempt={attempt} />
      )}
    </div>
  ) : null;

  // Layout based on mode
  const content = attempt ? (
    <ClickedElementsProvider attempt={attempt}>
      <ReviewProvider key={attempt.id}>
        <ExecutionProcessesProvider key={attempt.id} attemptId={attempt.id}>
          {mode === null ? (
            // Just attempt content
            <div className="h-full flex flex-col">
              {rightHeader}
              <div className="flex-1 min-h-0">
                {attemptContent}
              </div>
            </div>
          ) : (
            // Attempt + Aux split
            <div className="h-full flex flex-col">
              {rightHeader}
              <div className="flex-1 min-h-0 flex">
                <div className="flex-1 min-w-0">
                  {attemptContent}
                </div>
                <div className="flex-1 min-w-0 border-l">
                  {auxContent}
                </div>
              </div>
            </div>
          )}
        </ExecutionProcessesProvider>
      </ReviewProvider>
    </ClickedElementsProvider>
  ) : (
    <div className="h-full flex flex-col">
      {rightHeader}
      <div className="flex-1 min-h-0">
        {attemptContent}
      </div>
    </div>
  );

  return (
    <Dialog open={modal.visible} onOpenChange={handleClose} uncloseable>
      <DialogContent className="max-w-[95vw] max-h-[95vh] h-[95vh] overflow-hidden flex flex-col p-0">
        {/* Full content */}
        <div className="flex-1 min-h-0 overflow-hidden">
          {content}
        </div>
      </DialogContent>
    </Dialog>
  );
});

export { TaskDetailModal };
