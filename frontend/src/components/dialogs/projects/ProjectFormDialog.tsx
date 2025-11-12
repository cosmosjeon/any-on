import { useState } from 'react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { ProjectFormFields } from '@/components/projects/project-form-fields';
import { CreateProject, RepositoryInfo } from 'shared/types';
import { generateProjectNameFromPath } from '@/utils/string';
import NiceModal, { useModal } from '@ebay/nice-modal-react';
import { useProjectMutations } from '@/hooks/useProjectMutations';
import { useUserSystem } from '@/components/config-provider';

const INVALID_PROJECT_NAME_MESSAGE =
  'Project name cannot contain / \\ : * ? " < > | or control characters.';
const INVALID_PROJECT_NAME_CHARS = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

const containsInvalidProjectNameChars = (value: string) =>
  [...value].some(
    (char) => char.charCodeAt(0) < 32 || INVALID_PROJECT_NAME_CHARS.includes(char)
  );

export interface ProjectFormDialogProps {
  // No props needed - this is only for creating projects now
}

export type ProjectFormDialogResult = 'saved' | 'canceled';

export const ProjectFormDialog = NiceModal.create<ProjectFormDialogProps>(
  () => {
    const modal = useModal();
    const { config, githubTokenInvalid, githubSecretState, isCloud } =
      useUserSystem();
    const githubConnected = !!(
      isCloud &&
        config?.github?.username &&
        githubSecretState?.has_oauth_token &&
        !githubTokenInvalid
    );
    const [name, setName] = useState('');
    const [gitRepoPath, setGitRepoPath] = useState('');
    const [error, setError] = useState('');
    const [repoMode, setRepoMode] = useState<'existing' | 'new'>('new'); // Default to 'new' for Replit-style

    const { createProject, createProjectFromGithub } = useProjectMutations({
      onCreateSuccess: () => {
        modal.resolve('saved' as ProjectFormDialogResult);
        modal.hide();
      },
      onCreateError: (err) => {
        setError(err instanceof Error ? err.message : 'An error occurred');
      },
    });

    // Auto-populate project name from directory name
    const handleGitRepoPathChange = (path: string) => {
      setGitRepoPath(path);

      if (path) {
        const cleanName = generateProjectNameFromPath(path);
        if (cleanName) setName(cleanName);
      }
    };

    // Handle direct project creation from repo selection
    const handleDirectCreate = async (path: string, suggestedName: string) => {
      setError('');
      const finalName = suggestedName.trim();

      if (!finalName) {
        setError('Project name is required');
        return;
      }

      if (containsInvalidProjectNameChars(finalName)) {
        setError(INVALID_PROJECT_NAME_MESSAGE);
        return;
      }

      setName(finalName);

      const createData: CreateProject = {
        name: finalName,
        git_repo_path: path,
        use_existing_repo: true,
        setup_script: null,
        dev_script: null,
        cleanup_script: null,
        copy_files: null,
      };

      createProject.mutate(createData);
    };

    const handleGithubImport = (repo: RepositoryInfo) => {
      if (!githubConnected) {
        NiceModal.show('github-login');
        return;
      }
      setError('');
      createProjectFromGithub.mutate({
        repository_id: Number(repo.id),
        name: repo.name || repo.full_name,
        clone_url: repo.clone_url,
        setup_script: null,
        dev_script: null,
        cleanup_script: null,
      });
    };

    const handleConnectGithub = () => {
      NiceModal.show('github-login');
    };

    const handleSubmit = async (e: React.FormEvent) => {
      e.preventDefault();
      setError('');

      // Replit-style: For new projects, don't send git_repo_path (empty string)
      // Server will auto-generate from workspace_dir + project name
      const finalName = name.trim();
      if (!finalName) {
        setError('Project name is required');
        return;
      }

      if (containsInvalidProjectNameChars(finalName)) {
        setError(INVALID_PROJECT_NAME_MESSAGE);
        return;
      }

      setName(finalName);

      // Creating new project (Replit-style: no path needed)
      const createData: CreateProject = {
        name: finalName,
        git_repo_path: '', // Empty = server auto-generates path
        use_existing_repo: false, // Always create new repo
        setup_script: null,
        dev_script: null,
        cleanup_script: null,
        copy_files: null,
      };

      createProject.mutate(createData);
    };

    const handleCancel = () => {
      // Reset form
      setName('');
      setGitRepoPath('');
      setError('');

      modal.resolve('canceled' as ProjectFormDialogResult);
      modal.hide();
    };

    const handleOpenChange = (open: boolean) => {
      if (!open) {
        handleCancel();
      }
    };

    return (
      <Dialog open={modal.visible} onOpenChange={handleOpenChange}>
        <DialogContent className="overflow-x-hidden">
          <DialogHeader>
            <DialogTitle>Create Project</DialogTitle>
            <DialogDescription>Choose your repository source</DialogDescription>
          </DialogHeader>

          <div className="mx-auto w-full max-w-2xl overflow-x-hidden px-1">
            <form onSubmit={handleSubmit} className="space-y-4">
              <ProjectFormFields
                isEditing={false}
                repoMode={repoMode}
                setRepoMode={setRepoMode}
                gitRepoPath={gitRepoPath}
                handleGitRepoPathChange={handleGitRepoPathChange}
                setName={setName}
                name={name}
                setupScript=""
                setSetupScript={() => {}}
                devScript=""
                setDevScript={() => {}}
                cleanupScript=""
                setCleanupScript={() => {}}
                copyFiles=""
                setCopyFiles={() => {}}
                error={error}
                setError={setError}
                projectId={undefined}
                onCreateProject={handleDirectCreate}
                githubFeatureEnabled={isCloud}
                githubConnected={githubConnected}
                onConnectGithub={handleConnectGithub}
                onImportFromGithub={handleGithubImport}
                importingFromGithub={createProjectFromGithub.isPending}
              />
              {repoMode === 'new' && (
                <Button
                  type="submit"
                  disabled={createProject.isPending || !name.trim()}
                  className="w-full"
                >
                  {createProject.isPending ? 'Creating...' : 'Create Project'}
                </Button>
              )}
            </form>
          </div>
        </DialogContent>
      </Dialog>
    );
  }
);
