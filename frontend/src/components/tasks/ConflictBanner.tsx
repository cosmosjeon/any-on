import { AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import type { ConflictOp } from 'shared/types';
import { displayConflictOpLabel } from '@/lib/conflicts';

export type Props = Readonly<{
  attemptBranch: string | null;
  baseBranch?: string;
  conflictedFiles: readonly string[];
  onOpenEditor: () => void;
  onAbort: () => void;
  op?: ConflictOp | null;
  onResolve?: () => void;
  enableResolve: boolean;
  enableAbort: boolean;
}>;

const MAX_VISIBLE_FILES = 8;

function getOperationTitle(op?: ConflictOp | null): {
  full: string;
  lower: string;
} {
  const title = displayConflictOpLabel(op);
  return { full: title, lower: title.toLowerCase() };
}

function getVisibleFiles(
  files: readonly string[],
  max = MAX_VISIBLE_FILES
): { visible: string[]; total: number; hasMore: boolean } {
  const visible = files.slice(0, max);
  return {
    visible,
    total: files.length,
    hasMore: files.length > visible.length,
  };
}

export function ConflictBanner({
  attemptBranch,
  baseBranch,
  conflictedFiles,
  onOpenEditor,
  onAbort,
  op,
  onResolve,
  enableResolve,
  enableAbort,
}: Props) {
  const { full: opTitle, lower: opTitleLower } = getOperationTitle(op);
  const {
    visible: visibleFiles,
    total,
    hasMore,
  } = getVisibleFiles(conflictedFiles);

  const heading = attemptBranch
    ? `${opTitle} in progress: '${attemptBranch}' → '${baseBranch}'.`
    : 'A Git operation with merge conflicts is in progress.';

  return (
    <div
      className="flex flex-col gap-2 rounded-md border border-amber-500/40 bg-amber-500/10 p-3 text-amber-700 dark:text-amber-300"
      role="status"
      aria-live="polite"
    >
      <div className="flex items-start gap-2">
        <AlertCircle
          className="mt-0.5 h-4 w-4 text-amber-600 dark:text-amber-400"
          aria-hidden
        />
        <div className="text-sm leading-relaxed">
          <span>{heading}</span>{' '}
          <span>
            Follow-ups are allowed; some actions may be temporarily unavailable
            until you resolve the conflicts or abort the {opTitleLower}.
          </span>
          {visibleFiles.length > 0 && (
            <div className="mt-1 text-xs text-amber-700/90 dark:text-amber-300/80">
              <div className="font-medium">
                Conflicted files ({visibleFiles.length}
                {hasMore ? ` of ${total}` : ''}):
              </div>
              <div className="mt-1 grid grid-cols-1 gap-0.5">
                {visibleFiles.map((f) => (
                  <div key={f} className="truncate">
                    {f}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      <div className="flex flex-wrap gap-2">
        {onResolve && (
          <Button
            size="sm"
            onClick={onResolve}
            disabled={!enableResolve}
            className="bg-amber-500 text-white hover:bg-amber-600"
          >
            Resolve conflicts
          </Button>
        )}
        <Button
          size="sm"
          variant="outline"
          className="border-amber-500/40 text-amber-700 hover:bg-amber-500/10 dark:text-amber-300"
          onClick={onOpenEditor}
        >
          Open in Editor
        </Button>

        <Button
          size="sm"
          variant="outline"
          className="border-destructive/40 text-destructive hover:bg-destructive/10"
          onClick={onAbort}
          disabled={!enableAbort}
          aria-disabled={!enableAbort}
        >
          Abort {opTitle}
        </Button>
      </div>
    </div>
  );
}
