import { useCallback, useState } from 'react';
import { ChevronDown, ChevronRight } from 'lucide-react';
import type { TaskWithAttemptStatus } from 'shared/types';
import { TaskCard } from './TaskCard';
import { cn } from '@/lib/utils';

type Task = TaskWithAttemptStatus;

interface EpicCardProps {
  epic: Task;
  stories: Task[];
  projectId: string;
  onViewDetails: (task: Task) => void;
}

export function EpicCard({ epic, stories, projectId, onViewDetails }: EpicCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  const handleToggle = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    setIsExpanded(prev => !prev);
  }, []);

  return (
    <div className="space-y-1.5">
      {/* Epic Card */}
      <div className="bg-card border border-border/40 rounded-lg hover:border-border hover:shadow-lg transition-all duration-200">
        <div className="p-2">
          {/* Epic Header */}
          <div className="flex items-start gap-2 mb-1.5">
            <button
              onClick={handleToggle}
              className="flex-shrink-0 mt-0.5 hover:bg-accent rounded p-0.5 transition-colors"
              aria-label={isExpanded ? "Collapse stories" : "Expand stories"}
            >
              {isExpanded ? (
                <ChevronDown className="h-3 w-3 text-muted-foreground" />
              ) : (
                <ChevronRight className="h-3 w-3 text-muted-foreground" />
              )}
            </button>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-1.5 mb-1">
                <span className="text-[8px] font-semibold text-purple-600 bg-purple-100 px-1.5 py-0.5 rounded">
                  EPIC
                </span>
                {stories.length > 0 && (
                  <span className="text-[8px] text-muted-foreground">
                    {stories.length} {stories.length === 1 ? 'story' : 'stories'}
                  </span>
                )}
              </div>
              <TaskCard
                task={epic}
                projectId={projectId}
                onViewDetails={onViewDetails}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Stories */}
      {isExpanded && stories.length > 0 && (
        <div className="ml-6 space-y-1.5">
          {stories.map((story) => (
            <div
              key={story.id}
              className={cn(
                "bg-card border border-border/40 rounded-lg",
                "hover:border-border hover:shadow-lg transition-all duration-200",
                "relative before:absolute before:left-[-16px] before:top-1/2",
                "before:w-4 before:h-px before:bg-border"
              )}
            >
              <div className="p-2">
                <div className="flex items-center gap-1.5 mb-1">
                  <span className="text-[8px] font-semibold text-blue-600 bg-blue-100 px-1.5 py-0.5 rounded">
                    STORY
                  </span>
                </div>
                <TaskCard
                  task={story}
                  projectId={projectId}
                  onViewDetails={onViewDetails}
                />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
