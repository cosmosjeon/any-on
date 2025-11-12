import { create } from 'zustand';
import { ReviewDraft } from '@/contexts/ReviewProvider';

// UI state management store consolidating multiple contexts
interface UIState {
  // Process Selection (replaces ProcessSelectionContext)
  selectedProcessId: string | null;
  setSelectedProcessId: (id: string | null) => void;

  // Tab Navigation (replaces TabNavigationContext)
  selectedTab: string;
  setSelectedTab: (tab: string) => void;

  // Retry UI state (replaces RetryUiContext)
  retryingAttemptIds: Set<string>;
  setRetryingAttempt: (attemptId: string, isRetrying: boolean) => void;

  // Approval Forms (replaces ApprovalFormContext)
  approvalForms: Record<string, Record<string, unknown>>;
  setApprovalForm: (entryId: string, formData: Record<string, unknown>) => void;
  clearApprovalForm: (entryId: string) => void;

  // Review Drafts (part of ReviewProvider)
  reviewDrafts: Record<string, ReviewDraft>;
  setReviewDraft: (key: string, draft: ReviewDraft | null) => void;
  clearAllReviewDrafts: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  // Process Selection
  selectedProcessId: null,
  setSelectedProcessId: (id) => set({ selectedProcessId: id }),

  // Tab Navigation
  selectedTab: 'conversation',
  setSelectedTab: (tab) => set({ selectedTab: tab }),

  // Retry UI state
  retryingAttemptIds: new Set(),
  setRetryingAttempt: (attemptId, isRetrying) =>
    set((state) => {
      const newSet = new Set(state.retryingAttemptIds);
      if (isRetrying) {
        newSet.add(attemptId);
      } else {
        newSet.delete(attemptId);
      }
      return { retryingAttemptIds: newSet };
    }),

  // Approval Forms
  approvalForms: {},
  setApprovalForm: (entryId, formData) =>
    set((state) => ({
      approvalForms: {
        ...state.approvalForms,
        [entryId]: formData,
      },
    })),
  clearApprovalForm: (entryId) =>
    set((state) => {
      const newForms = { ...state.approvalForms };
      delete newForms[entryId];
      return { approvalForms: newForms };
    }),

  // Review Drafts
  reviewDrafts: {},
  setReviewDraft: (key, draft) =>
    set((state) => {
      if (draft === null) {
        const newDrafts = { ...state.reviewDrafts };
        delete newDrafts[key];
        return { reviewDrafts: newDrafts };
      }
      return {
        reviewDrafts: {
          ...state.reviewDrafts,
          [key]: draft,
        },
      };
    }),
  clearAllReviewDrafts: () => set({ reviewDrafts: {} }),
}));

// Convenience hooks for specific features
export const useProcessSelection = () =>
  useUIStore((state) => ({
    selectedProcessId: state.selectedProcessId,
    setSelectedProcessId: state.setSelectedProcessId,
  }));

export const useTabNavigation = () =>
  useUIStore((state) => ({
    selectedTab: state.selectedTab,
    setSelectedTab: state.setSelectedTab,
  }));

export const useRetryUI = () =>
  useUIStore((state) => ({
    retryingAttemptIds: state.retryingAttemptIds,
    setRetryingAttempt: state.setRetryingAttempt,
  }));

export const useApprovalForms = () =>
  useUIStore((state) => ({
    approvalForms: state.approvalForms,
    setApprovalForm: state.setApprovalForm,
    clearApprovalForm: state.clearApprovalForm,
  }));

export const useReviewDraftsStore = () =>
  useUIStore((state) => ({
    drafts: state.reviewDrafts,
    setDraft: state.setReviewDraft,
    clearAllDrafts: state.clearAllReviewDrafts,
  }));
