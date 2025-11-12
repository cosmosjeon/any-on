import { TaskStatus } from 'shared/types';

export const statusLabels: Record<TaskStatus, string> = {
  todo: 'Backlog',
  inprogress: 'Dev',
  inreview: 'Review',
  done: 'Done',
  cancelled: 'Conflict',
};

export const statusBoardColors: Record<TaskStatus, string> = {
  todo: 'hsl(240, 5%, 64%)', // Backlog - gray
  inprogress: 'hsl(221, 83%, 53%)', // Dev - blue
  inreview: 'hsl(38, 92%, 50%)', // Review - orange
  done: 'hsl(142, 76%, 36%)', // Done - green
  cancelled: 'hsl(0, 84%, 60%)', // Conflict - red
};

// Extended labels for frontend-only columns (like Plan)
export const extendedStatusLabels = {
  ...statusLabels,
  plan: 'Plan',
};

export const extendedStatusColors = {
  ...statusBoardColors,
  plan: 'hsl(280, 65%, 60%)', // Plan - purple
};
