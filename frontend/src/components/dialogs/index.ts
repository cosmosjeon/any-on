// Global app dialogs
export { DisclaimerDialog } from './global/DisclaimerDialog';
export { OnboardingDialog } from './global/OnboardingDialog';
export { PrivacyOptInDialog } from './global/PrivacyOptInDialog';
export { ReleaseNotesDialog } from './global/ReleaseNotesDialog';

// Authentication dialogs
export { GitHubLoginDialog } from './auth/GitHubLoginDialog';
export { ClaudeLoginDialog } from './auth/ClaudeLoginDialog';
export {
  ProvidePatDialog,
  type ProvidePatDialogProps,
} from './auth/ProvidePatDialog';

// Project-related dialogs
export {
  ProjectFormDialog,
  type ProjectFormDialogProps,
  type ProjectFormDialogResult,
} from './projects/ProjectFormDialog';

// Task-related dialogs
export {
  TaskDetailModal,
  type TaskDetailModalProps,
} from './tasks/TaskDetailModal';
export {
  TaskFormDialog,
  type TaskFormDialogProps,
} from './tasks/TaskFormDialog';

export { CreatePRDialog } from './tasks/CreatePRDialog';
export {
  DeleteTaskConfirmationDialog,
  type DeleteTaskConfirmationDialogProps,
} from './tasks/DeleteTaskConfirmationDialog';
export {
  TagEditDialog,
  type TagEditDialogProps,
  type TagEditResult,
} from './tasks/TagEditDialog';
export {
  ChangeTargetBranchDialog,
  type ChangeTargetBranchDialogProps,
  type ChangeTargetBranchDialogResult,
} from './tasks/ChangeTargetBranchDialog';
export {
  RebaseDialog,
  type RebaseDialogProps,
  type RebaseDialogResult,
} from './tasks/RebaseDialog';
export {
  RestoreLogsDialog,
  type RestoreLogsDialogProps,
  type RestoreLogsDialogResult,
} from './tasks/RestoreLogsDialog';
export {
  ViewProcessesDialog,
  type ViewProcessesDialogProps,
} from './tasks/ViewProcessesDialog';
export {
  GitActionsDialog,
  type GitActionsDialogProps,
} from './tasks/GitActionsDialog';

// Settings dialogs
export {
  CreateConfigurationDialog,
  type CreateConfigurationDialogProps,
  type CreateConfigurationResult,
} from './settings/CreateConfigurationDialog';
export {
  DeleteConfigurationDialog,
  type DeleteConfigurationDialogProps,
  type DeleteConfigurationResult,
} from './settings/DeleteConfigurationDialog';

// Shared/Generic dialogs
export { ConfirmDialog, type ConfirmDialogProps } from './shared/ConfirmDialog';
export {
  FolderPickerDialog,
  type FolderPickerDialogProps,
} from './shared/FolderPickerDialog';
