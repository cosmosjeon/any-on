import { useState, useEffect, useCallback } from 'react';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  AlertCircle,
  Folder,
  Search,
  FolderGit,
  FolderPlus,
  ArrowLeft,
  Github,
} from 'lucide-react';
import { useScriptPlaceholders } from '@/hooks/useScriptPlaceholders';
import { CopyFilesField } from './copy-files-field';
// Removed collapsible sections for simplicity; show fields always in edit mode
import { fileSystemApi, githubApi } from '@/lib/api';
import { showFolderPicker } from '@/lib/modals';
import { DirectoryEntry, RepositoryInfo } from 'shared/types';
import { generateProjectNameFromPath } from '@/utils/string';
import { Badge } from '@/components/ui/badge';

interface ProjectFormFieldsProps {
  isEditing: boolean;
  repoMode: 'existing' | 'new';
  setRepoMode: (mode: 'existing' | 'new') => void;
  gitRepoPath: string;
  handleGitRepoPathChange: (path: string) => void;
  parentPath: string;
  setParentPath: (path: string) => void;
  setFolderName: (name: string) => void;
  setName: (name: string) => void;
  name: string;
  setupScript: string;
  setSetupScript: (script: string) => void;
  devScript: string;
  setDevScript: (script: string) => void;
  cleanupScript: string;
  setCleanupScript: (script: string) => void;
  copyFiles: string;
  setCopyFiles: (files: string) => void;
  error: string;
  setError: (error: string) => void;
  projectId?: string;
  onCreateProject?: (path: string, name: string) => void;
  githubFeatureEnabled: boolean;
  githubConnected: boolean;
  onConnectGithub?: () => void;
  onImportFromGithub?: (repo: RepositoryInfo) => void;
  importingFromGithub?: boolean;
}

export function ProjectFormFields({
  isEditing,
  repoMode,
  setRepoMode,
  gitRepoPath,
  handleGitRepoPathChange,
  parentPath,
  setParentPath,
  setFolderName,
  setName,
  name,
  setupScript,
  setSetupScript,
  devScript,
  setDevScript,
  cleanupScript,
  setCleanupScript,
  copyFiles,
  setCopyFiles,
  error,
  setError,
  projectId,
  onCreateProject,
  githubFeatureEnabled,
  githubConnected,
  onConnectGithub,
  onImportFromGithub,
  importingFromGithub = false,
}: ProjectFormFieldsProps) {
  const placeholders = useScriptPlaceholders();

  // Repository loading state
  const [allRepos, setAllRepos] = useState<DirectoryEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [reposError, setReposError] = useState('');
  const [showMoreOptions, setShowMoreOptions] = useState(false);
  const [selectionView, setSelectionView] =
    useState<'options' | 'local' | 'github'>('options');
  const [githubRepos, setGithubRepos] = useState<RepositoryInfo[]>([]);
  const [githubLoading, setGithubLoading] = useState(false);
  const [githubError, setGithubError] = useState('');
  const [githubPage, setGithubPage] = useState(1);
  const [githubHasMore, setGithubHasMore] = useState(false);

  // Lazy-load repositories when the user navigates to the repo list
  useEffect(() => {
    if (
      !isEditing &&
      selectionView === 'local' &&
      !loading &&
      allRepos.length === 0
    ) {
      loadRecentRepos();
    }
  }, [isEditing, selectionView, loading, allRepos.length]);

const loadRecentRepos = async () => {
  setLoading(true);
  setReposError('');

    try {
      const discoveredRepos = await fileSystemApi.listGitRepos();
      setAllRepos(discoveredRepos);
    } catch (err) {
      setReposError('Failed to load repositories');
      console.error('Failed to load repos:', err);
    } finally {
      setLoading(false);
    }
  };

  const fetchGithubRepos = useCallback(
    async (page: number) => {
      if (!githubConnected) return;
      setGithubLoading(true);
      setGithubError('');

      try {
        const repos = await githubApi.listRepositories(page);
        setGithubRepos(repos);
        setGithubHasMore(repos.length === 50);
      } catch (err) {
        console.error('Failed to load GitHub repositories:', err);
        setGithubError(
          'Failed to load GitHub repositories. Please try again or reconnect your GitHub account.'
        );
      } finally {
        setGithubLoading(false);
      }
    },
    [githubConnected]
  );

  useEffect(() => {
    if (isEditing || selectionView !== 'github') {
      return;
    }
    if (!githubConnected) {
      setGithubError('Connect your GitHub account to view repositories.');
      setGithubRepos([]);
      return;
    }
    fetchGithubRepos(githubPage);
  }, [
    isEditing,
    selectionView,
    githubPage,
    githubConnected,
    fetchGithubRepos,
  ]);

  return (
    <>
      {!isEditing && repoMode === 'existing' && (
        <div className="space-y-4">
          {/* Show selection interface only when no repo is selected */}
          <>
            {/* Initial choice cards - Stage 1 */}
            {selectionView === 'options' && (
              <>
                {/* From Git Repository card */}
                <div
                  className="p-4 border cursor-pointer hover:shadow-md transition-shadow rounded-lg bg-card"
                  onClick={() => {
                    setSelectionView('local');
                    setError('');
                  }}
                >
                  <div className="flex items-start gap-3">
                    <FolderGit className="h-5 w-5 mt-0.5 flex-shrink-0 text-muted-foreground" />
                    <div className="min-w-0 flex-1">
                      <div className="font-medium text-foreground">
                        From Git Repository
                      </div>
                      <div className="text-xs text-muted-foreground mt-1">
                        Use an existing repository as your project base
                      </div>
                    </div>
                  </div>
                </div>

                {/* Create Blank Project card */}
                <div
                  className="p-4 border cursor-pointer hover:shadow-md transition-shadow rounded-lg bg-card"
                  onClick={() => {
                    setRepoMode('new');
                    setError('');
                  }}
                >
                  <div className="flex items-start gap-3">
                    <FolderPlus className="h-5 w-5 mt-0.5 flex-shrink-0 text-muted-foreground" />
                    <div className="min-w-0 flex-1">
                      <div className="font-medium text-foreground">
                        Create Blank Project
                      </div>
                      <div className="text-xs text-muted-foreground mt-1">
                        Start a new project from scratch
                      </div>
                    </div>
                  </div>
                </div>

                {githubFeatureEnabled && (
                  <div
                    className={`p-4 border rounded-lg bg-card transition-shadow ${
                      githubConnected
                        ? 'cursor-pointer hover:shadow-md'
                        : 'opacity-75'
                    }`}
                    onClick={() => {
                      if (!githubConnected) {
                        onConnectGithub?.();
                        return;
                      }
                      setGithubPage(1);
                      setSelectionView('github');
                      setError('');
                    }}
                  >
                    <div className="flex items-start gap-3">
                      <Github className="h-5 w-5 mt-0.5 flex-shrink-0 text-muted-foreground" />
                      <div className="min-w-0 flex-1">
                        <div className="font-medium text-foreground">
                          Import from GitHub
                        </div>
                        <div className="text-xs text-muted-foreground mt-1">
                          Browse repositories in your GitHub account and clone
                          them into Anyon
                        </div>
                        {!githubConnected && (
                          <div className="mt-3">
                            <Button
                              type="button"
                              size="sm"
                              variant="outline"
                              onClick={(e) => {
                                e.stopPropagation();
                                onConnectGithub?.();
                              }}
                            >
                              Connect GitHub
                            </Button>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                )}
              </>
            )}

            {/* Repository selection - Stage 2A */}
            {selectionView === 'local' && (
              <>
                {/* Back button */}
                <button
                  className="text-sm text-muted-foreground hover:text-foreground flex items-center gap-1 mb-4"
                  onClick={() => {
                    setSelectionView('options');
                    setError('');
                  }}
                >
                  <ArrowLeft className="h-3 w-3" />
                  Back to options
                </button>

                {/* Repository cards */}
                {!loading && allRepos.length > 0 && (
                  <div className="space-y-2">
                    {allRepos
                      .slice(0, showMoreOptions ? allRepos.length : 3)
                      .map((repo) => (
                        <div
                          key={repo.path}
                          className="p-4 border cursor-pointer hover:shadow-md transition-shadow rounded-lg bg-card"
                          onClick={() => {
                            setError('');
                            const cleanName = generateProjectNameFromPath(
                              repo.path
                            );
                            onCreateProject?.(repo.path, cleanName);
                          }}
                        >
                          <div className="flex items-start gap-3">
                            <FolderGit className="h-5 w-5 mt-0.5 flex-shrink-0 text-muted-foreground" />
                            <div className="min-w-0 flex-1">
                              <div className="font-medium text-foreground">
                                {repo.name}
                              </div>
                              <div className="text-xs text-muted-foreground truncate mt-1">
                                {repo.path}
                              </div>
                            </div>
                          </div>
                        </div>
                      ))}

                    {/* Show more/less for repositories */}
                    {!showMoreOptions && allRepos.length > 3 && (
                      <button
                        className="text-sm text-muted-foreground hover:text-foreground transition-colors text-left"
                        onClick={() => setShowMoreOptions(true)}
                      >
                        Show {allRepos.length - 3} more repositories
                      </button>
                    )}
                    {showMoreOptions && allRepos.length > 3 && (
                      <button
                        className="text-sm text-muted-foreground hover:text-foreground transition-colors text-left"
                        onClick={() => setShowMoreOptions(false)}
                      >
                        Show less
                      </button>
                    )}
                  </div>
                )}

                {/* Loading state */}
                {loading && (
                  <div className="p-4 border rounded-lg bg-card">
                    <div className="flex items-center gap-3">
                      <div className="animate-spin h-5 w-5 border-2 border-muted-foreground border-t-transparent rounded-full"></div>
                      <div className="text-sm text-muted-foreground">
                        Loading repositories...
                      </div>
                    </div>
                  </div>
                )}

                {/* Error state */}
                {!loading && reposError && (
                  <div className="p-4 border border-destructive rounded-lg bg-destructive/5">
                    <div className="flex items-center gap-3">
                      <AlertCircle className="h-5 w-5 text-destructive flex-shrink-0" />
                      <div className="text-sm text-destructive">
                        {reposError}
                      </div>
                    </div>
                  </div>
                )}

                {/* Browse for repository card */}
                <div
                  className="p-4 border border-dashed cursor-pointer hover:shadow-md transition-shadow rounded-lg bg-card"
                  onClick={async () => {
                    setError('');
                    const selectedPath = await showFolderPicker({
                      title: 'Select Git Repository',
                      description: 'Choose an existing git repository',
                    });
                    if (selectedPath) {
                      const projectName =
                        generateProjectNameFromPath(selectedPath);
                      if (onCreateProject) {
                        onCreateProject(selectedPath, projectName);
                      }
                    }
                  }}
                >
                  <div className="flex items-start gap-3">
                    <Search className="h-5 w-5 mt-0.5 flex-shrink-0 text-muted-foreground" />
                    <div className="min-w-0 flex-1">
                      <div className="font-medium text-foreground">
                        Search all repos
                      </div>
                      <div className="text-xs text-muted-foreground mt-1">
                        Browse and select any repository on your system
                      </div>
                    </div>
                  </div>
                </div>
              </>
            )}

            {selectionView === 'github' && (
              <div className="space-y-4">
                <button
                  className="text-sm text-muted-foreground hover:text-foreground flex items-center gap-1"
                  onClick={() => {
                    setSelectionView('options');
                    setGithubError('');
                  }}
                >
                  <ArrowLeft className="h-3 w-3" />
                  Back to options
                </button>

                {!githubConnected ? (
                  <div className="p-4 border rounded-lg bg-card">
                    <p className="text-sm text-muted-foreground mb-3">
                      Connect your GitHub account to list repositories.
                    </p>
                    <Button size="sm" onClick={onConnectGithub}>
                      Connect GitHub
                    </Button>
                  </div>
                ) : (
                  <>
                    {githubError && (
                      <Alert variant="destructive">
                        <AlertCircle className="h-4 w-4" />
                        <AlertDescription>{githubError}</AlertDescription>
                      </Alert>
                    )}

                    {githubLoading && (
                      <div className="p-4 border rounded-lg bg-card flex items-center gap-3 text-sm text-muted-foreground">
                        <div className="animate-spin h-5 w-5 border-2 border-muted-foreground border-t-transparent rounded-full" />
                        Loading GitHub repositories...
                      </div>
                    )}

                    {!githubLoading && !githubError && githubRepos.length === 0 && (
                      <div className="p-4 border rounded-lg bg-card text-sm text-muted-foreground">
                        No repositories found. You may need to grant access to more organizations.
                      </div>
                    )}

                    {!githubLoading && githubRepos.length > 0 && (
                      <div className="space-y-2">
                        {githubRepos.map((repo) => (
                          <div
                            key={`${repo.owner}/${repo.name}`}
                            className="p-4 border rounded-lg bg-card"
                          >
                            <div className="flex items-start justify-between gap-3">
                              <div className="min-w-0 flex-1">
                                <div className="font-medium text-foreground truncate">
                                  {repo.full_name || repo.name}
                                </div>
                                <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
                                  {repo.description || 'No description provided.'}
                                </p>
                              </div>
                              <Badge variant={repo.private ? 'secondary' : 'outline'}>
                                {repo.private ? 'Private' : 'Public'}
                              </Badge>
                            </div>
                            <div className="flex flex-wrap items-center justify-between gap-3 mt-4">
                              <span className="text-xs text-muted-foreground">
                                Default branch: {repo.default_branch}
                              </span>
                              <Button
                                size="sm"
                                disabled={importingFromGithub}
                                onClick={() => onImportFromGithub?.(repo)}
                              >
                                {importingFromGithub ? 'Importing…' : 'Import'}
                              </Button>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}

                    <div className="flex items-center justify-between pt-2">
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        disabled={githubPage === 1 || githubLoading}
                        onClick={() => setGithubPage((prev) => Math.max(1, prev - 1))}
                      >
                        Previous
                      </Button>
                      <span className="text-xs text-muted-foreground">
                        Page {githubPage}
                      </span>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        disabled={!githubHasMore || githubLoading}
                        onClick={() => setGithubPage((prev) => prev + 1)}
                      >
                        Next
                      </Button>
                    </div>
                  </>
                )}
              </div>
            )}
          </>
        </div>
      )}

      {/* Blank Project Form */}
      {!isEditing && repoMode === 'new' && (
        <div className="space-y-4">
          {/* Back button */}
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => {
              setRepoMode('existing');
              setSelectionView('options');
              setError('');
              setName('');
              setParentPath('');
              setFolderName('');
            }}
            className="flex items-center gap-2"
          >
            <ArrowLeft className="h-4 w-4" />
            Back to options
          </Button>

          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="new-project-name">
                Project Name <span className="text-red-500">*</span>
              </Label>
              <Input
                id="new-project-name"
                type="text"
                value={name}
                onChange={(e) => {
                  setName(e.target.value);
                  if (e.target.value) {
                    setFolderName(
                      e.target.value
                        .toLowerCase()
                        .replace(/\s+/g, '-')
                        .replace(/[^a-z0-9-]/g, '')
                    );
                  }
                }}
                placeholder="My Awesome Project"
                className="placeholder:text-secondary-foreground placeholder:opacity-100"
                required
              />
              <p className="text-xs text-muted-foreground">
                The folder name will be auto-generated from the project name
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="parent-path">Parent Directory</Label>
              <div className="flex space-x-2">
                <Input
                  id="parent-path"
                  type="text"
                  value={parentPath}
                  onChange={(e) => setParentPath(e.target.value)}
                  placeholder="Current Directory"
                  className="flex-1 placeholder:text-secondary-foreground placeholder:opacity-100"
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onClick={async () => {
                    const selectedPath = await showFolderPicker({
                      title: 'Select Parent Directory',
                      description: 'Choose where to create the new repository',
                      value: parentPath,
                    });
                    if (selectedPath) {
                      setParentPath(selectedPath);
                    }
                  }}
                >
                  <Folder className="h-4 w-4" />
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                Leave empty to use your current working directory, or specify a
                custom path.
              </p>
            </div>
          </div>
        </div>
      )}

      {isEditing && (
        <>
          <div className="space-y-2">
            <Label htmlFor="git-repo-path">Git Repository Path</Label>
            <div className="flex space-x-2">
              <Input
                id="git-repo-path"
                type="text"
                value={gitRepoPath}
                onChange={(e) => handleGitRepoPathChange(e.target.value)}
                placeholder="/path/to/your/existing/repo"
                required
                className="flex-1"
              />
              <Button
                type="button"
                variant="outline"
                onClick={async () => {
                  const selectedPath = await showFolderPicker({
                    title: 'Select Git Repository',
                    description: 'Choose an existing git repository',
                    value: gitRepoPath,
                  });
                  if (selectedPath) {
                    handleGitRepoPathChange(selectedPath);
                  }
                }}
              >
                <Folder className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="name">Project Name</Label>
            <Input
              id="name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter project name"
              required
            />
          </div>
        </>
      )}

      {isEditing && (
        <div className="space-y-4 pt-4 border-t border-border">
          <div className="space-y-2">
            <Label htmlFor="setup-script">Setup Script</Label>
            <textarea
              id="setup-script"
              value={setupScript}
              onChange={(e) => setSetupScript(e.target.value)}
              placeholder={placeholders.setup}
              rows={4}
              className="w-full px-3 py-2 text-sm border border-input bg-background text-foreground rounded-md resize-vertical focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <p className="text-sm text-muted-foreground">
              This script will run after creating the worktree and before the
              coding agent starts. Use it for setup tasks like installing
              dependencies or preparing the environment.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="dev-script">Dev Server Script</Label>
            <textarea
              id="dev-script"
              value={devScript}
              onChange={(e) => setDevScript(e.target.value)}
              placeholder={placeholders.dev}
              rows={4}
              className="w-full px-3 py-2 text-sm border border-input bg-background text-foreground rounded-md resize-vertical focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <p className="text-sm text-muted-foreground">
              This script can be run from task attempts to start a development
              server. Use it to quickly start your project's dev server for
              testing changes.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="cleanup-script">Cleanup Script</Label>
            <textarea
              id="cleanup-script"
              value={cleanupScript}
              onChange={(e) => setCleanupScript(e.target.value)}
              placeholder={placeholders.cleanup}
              rows={4}
              className="w-full px-3 py-2 text-sm border border-input bg-background text-foreground rounded-md resize-vertical focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <p className="text-sm text-muted-foreground">
              This script runs after coding agent execution{' '}
              <strong>only if changes were made</strong>. Use it for quality
              assurance tasks like running linters, formatters, tests, or other
              validation steps. If no changes are made, this script is skipped.
            </p>
          </div>

          <div className="space-y-2">
            <Label>Copy Files</Label>
            <CopyFilesField
              value={copyFiles}
              onChange={setCopyFiles}
              projectId={projectId}
            />
            <p className="text-sm text-muted-foreground">
              Comma-separated list of files to copy from the original project
              directory to the worktree. These files will be copied after the
              worktree is created but before the setup script runs. Useful for
              environment-specific files like .env, configuration files, and
              local settings. Make sure these are gitignored or they could get
              committed!
            </p>
          </div>
        </div>
      )}

      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
    </>
  );
}
