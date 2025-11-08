import { useCallback, useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { cloneDeep, merge, isEqual } from 'lodash';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Checkbox } from '@/components/ui/checkbox';
import { ChevronDown, Key, Loader2, Sparkles, Volume2 } from 'lucide-react';
import {
  BaseCodingAgent,
  EditorType,
  ExecutorProfileId,
  SoundFile,
  ThemeMode,
  UiLanguage,
} from 'shared/types';
import { getLanguageOptions } from '@/i18n/languages';

import { toPrettyCase } from '@/utils/string';
import { useTheme } from '@/components/theme-provider';
import { useUserSystem } from '@/components/config-provider';
import { TagManager } from '@/components/TagManager';
import NiceModal from '@ebay/nice-modal-react';
import { claudeAuthApi } from '@/lib/api';

export function GeneralSettings() {
  const { t } = useTranslation(['settings', 'common']);

  // Get language options with proper display names
  const languageOptions = getLanguageOptions(
    t('language.browserDefault', {
      ns: 'common',
      defaultValue: 'Browser Default',
    })
  );
  const {
    config,
    loading,
    updateAndSaveConfig, // Use this on Save
    profiles,
    githubSecretState,
    claudeSecretState,
    isCloud,
    reloadSystem,
  } = useUserSystem();

  // Draft state management
  const [draft, setDraft] = useState(() => (config ? cloneDeep(config) : null));
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [branchPrefixError, setBranchPrefixError] = useState<string | null>(
    null
  );
  const { setTheme } = useTheme();
  const [claudeActionLoading, setClaudeActionLoading] = useState(false);
  const [claudeActionType, setClaudeActionType] = useState<
    'connect' | 'disconnect' | null
  >(null);

  const validateBranchPrefix = useCallback(
    (prefix: string): string | null => {
      if (!prefix) return null; // empty allowed
      if (prefix.includes('/'))
        return t('settings.general.git.branchPrefix.errors.slash');
      if (prefix.startsWith('.'))
        return t('settings.general.git.branchPrefix.errors.startsWithDot');
      if (prefix.endsWith('.') || prefix.endsWith('.lock'))
        return t('settings.general.git.branchPrefix.errors.endsWithDot');
      if (prefix.includes('..') || prefix.includes('@{'))
        return t('settings.general.git.branchPrefix.errors.invalidSequence');
      if (/[ \t~^:?*[\\]/.test(prefix))
        return t('settings.general.git.branchPrefix.errors.invalidChars');
      // Control chars check
      for (let i = 0; i < prefix.length; i++) {
        const code = prefix.charCodeAt(i);
        if (code < 0x20 || code === 0x7f)
          return t('settings.general.git.branchPrefix.errors.controlChars');
      }
      return null;
    },
    [t]
  );

  // When config loads or changes externally, update draft only if not dirty
  useEffect(() => {
    if (!config) return;
    if (!dirty) {
      setDraft(cloneDeep(config));
    }
  }, [config, dirty]);

  // Check for unsaved changes
  const hasUnsavedChanges = useMemo(() => {
    if (!draft || !config) return false;
    return !isEqual(draft, config);
  }, [draft, config]);

  // Generic draft update helper
  const updateDraft = useCallback(
    (patch: Partial<typeof config>) => {
      setDraft((prev: typeof config) => {
        if (!prev) return prev;
        const next = merge({}, prev, patch);
        // Mark dirty if changed
        if (!isEqual(next, config)) {
          setDirty(true);
        }
        return next;
      });
    },
    [config]
  );

  // Optional: warn on tab close/navigation with unsaved changes
  useEffect(() => {
    const handler = (e: BeforeUnloadEvent) => {
      if (hasUnsavedChanges) {
        e.preventDefault();
        e.returnValue = '';
      }
    };
    window.addEventListener('beforeunload', handler);
    return () => window.removeEventListener('beforeunload', handler);
  }, [hasUnsavedChanges]);

  const playSound = async (soundFile: SoundFile) => {
    const audio = new Audio(`/api/sounds/${soundFile}`);
    try {
      await audio.play();
    } catch (err) {
      console.error('Failed to play sound:', err);
    }
  };

  const handleSave = async () => {
    if (!draft) return;

    setSaving(true);
    setError(null);
    setSuccess(false);

    try {
      await updateAndSaveConfig(draft); // Atomically apply + persist
      setTheme(draft.theme);
      setDirty(false);
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(t('settings.general.save.error'));
      console.error('Error saving config:', err);
    } finally {
      setSaving(false);
    }
  };

  const handleDiscard = () => {
    if (!config) return;
    setDraft(cloneDeep(config));
    setDirty(false);
  };

  const resetDisclaimer = async () => {
    if (!config) return;
    updateAndSaveConfig({ disclaimer_acknowledged: false });
  };

  const resetOnboarding = async () => {
    if (!config) return;
    updateAndSaveConfig({ onboarding_acknowledged: false });
  };

  const githubFeatureEnabled = isCloud;
  const githubConnected = !!(
    githubFeatureEnabled &&
    config?.github?.username &&
    githubSecretState?.has_oauth_token
  );
  const patConfigured = !!githubSecretState?.has_pat;
  const claudeFeatureEnabled = isCloud;
  const claudeConnected = !!(
    claudeFeatureEnabled && claudeSecretState?.has_credentials
  );

  const handleLogout = useCallback(async () => {
    if (!config) return;
    updateAndSaveConfig({
      github: {
        ...config.github,
        oauth_token: '',
        username: null,
        primary_email: null,
      },
    });
  }, [config, updateAndSaveConfig]);

  const handleRemovePat = useCallback(async () => {
    if (!config) return;
    updateAndSaveConfig({
      github: {
        ...config.github,
        pat: '',
      },
    });
  }, [config, updateAndSaveConfig]);

  const handleClaudeLogin = useCallback(async () => {
    if (!claudeFeatureEnabled) return;
    setClaudeActionType('connect');
    setClaudeActionLoading(true);
    setError(null);
    try {
      const result = await NiceModal.show('claude-login');
      if (result) {
        await reloadSystem();
      }
    } catch (err) {
      console.error('Failed to start Claude login', err);
      setError(t('settings.general.claude.errors.start'));
    } finally {
      setClaudeActionLoading(false);
      setClaudeActionType(null);
    }
  }, [claudeFeatureEnabled, reloadSystem, t]);

  const handleClaudeLogout = useCallback(async () => {
    if (!claudeFeatureEnabled) return;
    setClaudeActionType('disconnect');
    setClaudeActionLoading(true);
    try {
      await claudeAuthApi.logout();
      await reloadSystem();
    } catch (err) {
      console.error('Failed to remove Claude credentials', err);
      setError(t('settings.general.claude.errors.logout'));
    } finally {
      setClaudeActionLoading(false);
      setClaudeActionType(null);
    }
  }, [claudeFeatureEnabled, reloadSystem, t]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="h-8 w-8 animate-spin" />
        <span className="ml-2">{t('settings.general.loading')}</span>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="py-8">
        <Alert variant="destructive">
          <AlertDescription>{t('settings.general.loadError')}</AlertDescription>
        </Alert>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {success && (
        <Alert className="border-green-200 bg-green-50 text-green-800 dark:border-green-800 dark:bg-green-950 dark:text-green-200">
          <AlertDescription className="font-medium">
            {t('settings.general.save.success')}
          </AlertDescription>
        </Alert>
      )}

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.general.appearance.title')}</CardTitle>
          <CardDescription>
            {t('settings.general.appearance.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="theme">
              {t('settings.general.appearance.theme.label')}
            </Label>
            <Select
              value={draft?.theme}
              onValueChange={(value: ThemeMode) =>
                updateDraft({ theme: value })
              }
            >
              <SelectTrigger id="theme">
                <SelectValue
                  placeholder={t(
                    'settings.general.appearance.theme.placeholder'
                  )}
                />
              </SelectTrigger>
              <SelectContent>
                {Object.values(ThemeMode).map((theme) => (
                  <SelectItem key={theme} value={theme}>
                    {toPrettyCase(theme)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-sm text-muted-foreground">
              {t('settings.general.appearance.theme.helper')}
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="language">
              {t('settings.general.appearance.language.label')}
            </Label>
            <Select
              value={draft?.language}
              onValueChange={(value: UiLanguage) =>
                updateDraft({ language: value })
              }
            >
              <SelectTrigger id="language">
                <SelectValue
                  placeholder={t(
                    'settings.general.appearance.language.placeholder'
                  )}
                />
              </SelectTrigger>
              <SelectContent>
                {languageOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-sm text-muted-foreground">
              {t('settings.general.appearance.language.helper')}
            </p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.general.taskExecution.title')}</CardTitle>
          <CardDescription>
            {t('settings.general.taskExecution.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="executor">
              {t('settings.general.taskExecution.executor.label')}
            </Label>
            <div className="grid grid-cols-2 gap-2">
              <Select
                value={draft?.executor_profile?.executor ?? ''}
                onValueChange={(value: string) => {
                  const variants = profiles?.[value];
                  const keepCurrentVariant =
                    variants &&
                    draft?.executor_profile?.variant &&
                    variants[draft.executor_profile.variant];

                  const newProfile: ExecutorProfileId = {
                    executor: value as BaseCodingAgent,
                    variant: keepCurrentVariant
                      ? draft!.executor_profile!.variant
                      : null,
                  };
                  updateDraft({
                    executor_profile: newProfile,
                  });
                }}
                disabled={!profiles}
              >
                <SelectTrigger id="executor">
                  <SelectValue
                    placeholder={t(
                      'settings.general.taskExecution.executor.placeholder'
                    )}
                  />
                </SelectTrigger>
                <SelectContent>
                  {profiles &&
                    Object.entries(profiles)
                      .sort((a, b) => a[0].localeCompare(b[0]))
                      .map(([profileKey]) => (
                        <SelectItem key={profileKey} value={profileKey}>
                          {profileKey}
                        </SelectItem>
                      ))}
                </SelectContent>
              </Select>

              {/* Show variant selector if selected profile has variants */}
              {(() => {
                const currentProfileVariant = draft?.executor_profile;
                const selectedProfile =
                  profiles?.[currentProfileVariant?.executor || ''];
                const hasVariants =
                  selectedProfile && Object.keys(selectedProfile).length > 0;

                if (hasVariants) {
                  return (
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button
                          variant="outline"
                          className="w-full h-10 px-2 flex items-center justify-between"
                        >
                          <span className="text-sm truncate flex-1 text-left">
                            {currentProfileVariant?.variant ||
                              t('settings.general.taskExecution.defaultLabel')}
                          </span>
                          <ChevronDown className="h-4 w-4 ml-1 flex-shrink-0" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent>
                        {Object.entries(selectedProfile).map(
                          ([variantLabel]) => (
                            <DropdownMenuItem
                              key={variantLabel}
                              onClick={() => {
                                const newProfile: ExecutorProfileId = {
                                  executor: currentProfileVariant!.executor,
                                  variant: variantLabel,
                                };
                                updateDraft({
                                  executor_profile: newProfile,
                                });
                              }}
                              className={
                                currentProfileVariant?.variant === variantLabel
                                  ? 'bg-accent'
                                  : ''
                              }
                            >
                              {variantLabel}
                            </DropdownMenuItem>
                          )
                        )}
                      </DropdownMenuContent>
                    </DropdownMenu>
                  );
                } else if (selectedProfile) {
                  // Show disabled button when profile exists but has no variants
                  return (
                    <Button
                      variant="outline"
                      className="w-full h-10 px-2 flex items-center justify-between"
                      disabled
                    >
                      <span className="text-sm truncate flex-1 text-left">
                        {t('settings.general.taskExecution.defaultLabel')}
                      </span>
                    </Button>
                  );
                }
                return null;
              })()}
            </div>
            <p className="text-sm text-muted-foreground">
              {t('settings.general.taskExecution.executor.helper')}
            </p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.general.editor.title')}</CardTitle>
          <CardDescription>
            {t('settings.general.editor.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="editor-type">
              {t('settings.general.editor.type.label')}
            </Label>
            <Select
              value={draft?.editor.editor_type}
              onValueChange={(value: EditorType) =>
                updateDraft({
                  editor: { ...draft!.editor, editor_type: value },
                })
              }
            >
              <SelectTrigger id="editor-type">
                <SelectValue
                  placeholder={t('settings.general.editor.type.placeholder')}
                />
              </SelectTrigger>
              <SelectContent>
                {Object.values(EditorType).map((editor) => (
                  <SelectItem key={editor} value={editor}>
                    {toPrettyCase(editor)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-sm text-muted-foreground">
              {t('settings.general.editor.type.helper')}
            </p>
          </div>

          {draft?.editor.editor_type === EditorType.CUSTOM && (
            <div className="space-y-2">
              <Label htmlFor="custom-command">
                {t('settings.general.editor.customCommand.label')}
              </Label>
              <Input
                id="custom-command"
                placeholder={t(
                  'settings.general.editor.customCommand.placeholder'
                )}
                value={draft?.editor.custom_command || ''}
                onChange={(e) =>
                  updateDraft({
                    editor: {
                      ...draft!.editor,
                      custom_command: e.target.value || null,
                    },
                  })
                }
              />
              <p className="text-sm text-muted-foreground">
                {t('settings.general.editor.customCommand.helper')}
              </p>
            </div>
          )}

          {(draft?.editor.editor_type === EditorType.VS_CODE ||
            draft?.editor.editor_type === EditorType.CURSOR ||
            draft?.editor.editor_type === EditorType.WINDSURF) && (
            <>
              <div className="space-y-2">
                <Label htmlFor="remote-ssh-host">
                  {t('settings.general.editor.remoteSsh.host.label')}
                </Label>
                <Input
                  id="remote-ssh-host"
                  placeholder={t(
                    'settings.general.editor.remoteSsh.host.placeholder'
                  )}
                  value={draft?.editor.remote_ssh_host || ''}
                  onChange={(e) =>
                    updateDraft({
                      editor: {
                        ...draft!.editor,
                        remote_ssh_host: e.target.value || null,
                      },
                    })
                  }
                />
                <p className="text-sm text-muted-foreground">
                  {t('settings.general.editor.remoteSsh.host.helper')}
                </p>
              </div>

              {draft?.editor.remote_ssh_host && (
                <div className="space-y-2">
                  <Label htmlFor="remote-ssh-user">
                    {t('settings.general.editor.remoteSsh.user.label')}
                  </Label>
                  <Input
                    id="remote-ssh-user"
                    placeholder={t(
                      'settings.general.editor.remoteSsh.user.placeholder'
                    )}
                    value={draft?.editor.remote_ssh_user || ''}
                    onChange={(e) =>
                      updateDraft({
                        editor: {
                          ...draft!.editor,
                          remote_ssh_user: e.target.value || null,
                        },
                      })
                    }
                  />
                  <p className="text-sm text-muted-foreground">
                    {t('settings.general.editor.remoteSsh.user.helper')}
                  </p>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Key className="h-5 w-5" />
            {t('settings.general.github.title')}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {!githubFeatureEnabled ? (
            <Alert>
              <AlertDescription>
                {t('settings.general.github.cloudOnly')}
              </AlertDescription>
            </Alert>
          ) : (
            <>
              {githubConnected ? (
                <div className="space-y-4">
                  <div className="flex items-center justify-between p-4 border rounded-lg">
                    <div>
                      <p className="font-medium">
                        {t('settings.general.github.connected', {
                          username: config.github.username,
                        })}
                      </p>
                      {config.github.primary_email && (
                        <p className="text-sm text-muted-foreground">
                          {config.github.primary_email}
                        </p>
                      )}
                    </div>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="outline" size="sm">
                          {t('settings.general.github.manage')}{' '}
                          <ChevronDown className="ml-1 h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={handleLogout}>
                          {t('settings.general.github.disconnect')}
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </div>
              ) : (
                <div className="space-y-4">
                  <p className="text-sm text-muted-foreground">
                    {t('settings.general.github.helper')}
                  </p>
                  <Button onClick={() => NiceModal.show('github-login')}>
                    {t('settings.general.github.connectButton')}
                  </Button>
                </div>
              )}

              <div className="space-y-2">
                <Label htmlFor="github-token">
                  {t('settings.general.github.pat.label')}
                </Label>
                <Input
                  id="github-token"
                  type="password"
                  placeholder={t('settings.general.github.pat.placeholder')}
                  value={draft?.github.pat || ''}
                  onChange={(e) =>
                    updateDraft({
                      github: {
                        ...draft!.github,
                        pat: e.target.value || null,
                      },
                    })
                  }
                />
                <p className="text-sm text-muted-foreground">
                  {t('settings.general.github.pat.helper')}{' '}
                  <a
                    href="https://github.com/settings/tokens"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-600 hover:underline"
                  >
                    {t('settings.general.github.pat.createTokenLink')}
                  </a>
                </p>
                {patConfigured && (
                  <div className="flex items-center justify-between rounded-md border bg-muted/30 px-3 py-2 text-sm text-muted-foreground">
                    <span>
                      {t('settings.general.github.pat.saved')}
                    </span>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={handleRemovePat}
                    >
                      {t('settings.general.github.pat.remove')}
                    </Button>
                  </div>
                )}
              </div>
            </>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Sparkles className="h-5 w-5" />
            {t('settings.general.claude.title')}
          </CardTitle>
          <CardDescription>
            {t('settings.general.claude.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {!claudeFeatureEnabled ? (
            <Alert>
              <AlertDescription>
                {t('settings.general.claude.cloudOnly')}
              </AlertDescription>
            </Alert>
          ) : (
            <>
              <div className="flex flex-col gap-4 rounded-lg border p-4 md:flex-row md:items-center md:justify-between">
                <div>
                  <p className="font-medium">
                    {claudeConnected
                      ? t('settings.general.claude.status.connected')
                      : t('settings.general.claude.status.disconnected')}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    {t('settings.general.claude.helper')}
                  </p>
                </div>
                <div className="flex flex-wrap gap-2">
                  {claudeConnected && (
                    <Button
                      variant="outline"
                      disabled={claudeActionLoading}
                      onClick={handleClaudeLogout}
                      className="flex items-center gap-2"
                    >
                      {claudeActionLoading && claudeActionType === 'disconnect' && (
                        <Loader2 className="h-4 w-4 animate-spin" />
                      )}
                      {claudeActionLoading && claudeActionType === 'disconnect'
                        ? t('settings.general.claude.actions.disconnecting')
                        : t('settings.general.claude.actions.disconnect')}
                    </Button>
                  )}
                  <Button
                    disabled={claudeActionLoading}
                    onClick={handleClaudeLogin}
                    className="flex items-center gap-2"
                  >
                    {claudeActionLoading && claudeActionType === 'connect' && (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    )}
                    {claudeActionLoading && claudeActionType === 'connect'
                      ? t('settings.general.claude.actions.connecting')
                      : claudeConnected
                        ? t('settings.general.claude.actions.reconnect')
                        : t('settings.general.claude.actions.connect')}
                  </Button>
                </div>
              </div>

              <div className="rounded-md border bg-muted/40 px-4 py-3 text-sm text-muted-foreground">
                <p className="mb-2 font-semibold">
                  {t('settings.general.claude.steps.title')}
                </p>
                <ol className="list-decimal space-y-1 pl-4">
                  <li>{t('settings.general.claude.steps.pick')}</li>
                  <li>{t('settings.general.claude.steps.browser')}</li>
                  <li>{t('settings.general.claude.steps.done')}</li>
                </ol>
              </div>
            </>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.general.git.title')}</CardTitle>
          <CardDescription>
            {t('settings.general.git.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="git-branch-prefix">
              {t('settings.general.git.branchPrefix.label')}
            </Label>
            <Input
              id="git-branch-prefix"
              type="text"
              placeholder={t('settings.general.git.branchPrefix.placeholder')}
              value={draft?.git_branch_prefix ?? ''}
              onChange={(e) => {
                const value = e.target.value.trim();
                updateDraft({ git_branch_prefix: value });
                setBranchPrefixError(validateBranchPrefix(value));
              }}
              aria-invalid={!!branchPrefixError}
              className={branchPrefixError ? 'border-destructive' : undefined}
            />
            {branchPrefixError && (
              <p className="text-sm text-destructive">{branchPrefixError}</p>
            )}
            <p className="text-sm text-muted-foreground">
              {t('settings.general.git.branchPrefix.helper')}{' '}
              {draft?.git_branch_prefix ? (
                <>
                  {t('settings.general.git.branchPrefix.preview')}{' '}
                  <code className="text-xs bg-muted px-1 py-0.5 rounded">
                    {t('settings.general.git.branchPrefix.previewWithPrefix', {
                      prefix: draft.git_branch_prefix,
                    })}
                  </code>
                </>
              ) : (
                <>
                  {t('settings.general.git.branchPrefix.preview')}{' '}
                  <code className="text-xs bg-muted px-1 py-0.5 rounded">
                    {t('settings.general.git.branchPrefix.previewNoPrefix')}
                  </code>
                </>
              )}
            </p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.general.notifications.title')}</CardTitle>
          <CardDescription>
            {t('settings.general.notifications.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center space-x-2">
            <Checkbox
              id="sound-enabled"
              checked={draft?.notifications.sound_enabled}
              onCheckedChange={(checked: boolean) =>
                updateDraft({
                  notifications: {
                    ...draft!.notifications,
                    sound_enabled: checked,
                  },
                })
              }
            />
            <div className="space-y-0.5">
              <Label htmlFor="sound-enabled" className="cursor-pointer">
                {t('settings.general.notifications.sound.label')}
              </Label>
              <p className="text-sm text-muted-foreground">
                {t('settings.general.notifications.sound.helper')}
              </p>
            </div>
          </div>
          {draft?.notifications.sound_enabled && (
            <div className="ml-6 space-y-2">
              <Label htmlFor="sound-file">
                {t('settings.general.notifications.sound.fileLabel')}
              </Label>
              <div className="flex gap-2">
                <Select
                  value={draft.notifications.sound_file}
                  onValueChange={(value: SoundFile) =>
                    updateDraft({
                      notifications: {
                        ...draft.notifications,
                        sound_file: value,
                      },
                    })
                  }
                >
                  <SelectTrigger id="sound-file" className="flex-1">
                    <SelectValue
                      placeholder={t(
                        'settings.general.notifications.sound.filePlaceholder'
                      )}
                    />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.values(SoundFile).map((soundFile) => (
                      <SelectItem key={soundFile} value={soundFile}>
                        {toPrettyCase(soundFile)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => playSound(draft.notifications.sound_file)}
                  className="px-3"
                >
                  <Volume2 className="h-4 w-4" />
                </Button>
              </div>
              <p className="text-sm text-muted-foreground">
                {t('settings.general.notifications.sound.fileHelper')}
              </p>
            </div>
          )}
          <div className="flex items-center space-x-2">
            <Checkbox
              id="push-notifications"
              checked={draft?.notifications.push_enabled}
              onCheckedChange={(checked: boolean) =>
                updateDraft({
                  notifications: {
                    ...draft!.notifications,
                    push_enabled: checked,
                  },
                })
              }
            />
            <div className="space-y-0.5">
              <Label htmlFor="push-notifications" className="cursor-pointer">
                {t('settings.general.notifications.push.label')}
              </Label>
              <p className="text-sm text-muted-foreground">
                {t('settings.general.notifications.push.helper')}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.general.privacy.title')}</CardTitle>
          <CardDescription>
            {t('settings.general.privacy.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center space-x-2">
            <Checkbox
              id="analytics-enabled"
              checked={draft?.analytics_enabled ?? false}
              onCheckedChange={(checked: boolean) =>
                updateDraft({ analytics_enabled: checked })
              }
            />
            <div className="space-y-0.5">
              <Label htmlFor="analytics-enabled" className="cursor-pointer">
                {t('settings.general.privacy.telemetry.label')}
              </Label>
              <p className="text-sm text-muted-foreground">
                {t('settings.general.privacy.telemetry.helper')}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.general.taskTemplates.title')}</CardTitle>
          <CardDescription>
            {t('settings.general.taskTemplates.description')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <TagManager />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.general.safety.title')}</CardTitle>
          <CardDescription>
            {t('settings.general.safety.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium">
                {t('settings.general.safety.disclaimer.title')}
              </p>
              <p className="text-sm text-muted-foreground">
                {t('settings.general.safety.disclaimer.description')}
              </p>
            </div>
            <Button variant="outline" onClick={resetDisclaimer}>
              {t('settings.general.safety.disclaimer.button')}
            </Button>
          </div>
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium">
                {t('settings.general.safety.onboarding.title')}
              </p>
              <p className="text-sm text-muted-foreground">
                {t('settings.general.safety.onboarding.description')}
              </p>
            </div>
            <Button variant="outline" onClick={resetOnboarding}>
              {t('settings.general.safety.onboarding.button')}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Sticky Save Button */}
      <div className="sticky bottom-0 z-10 bg-background/80 backdrop-blur-sm border-t py-4">
        <div className="flex items-center justify-between">
          {hasUnsavedChanges ? (
            <span className="text-sm text-muted-foreground">
              {t('settings.general.save.unsavedChanges')}
            </span>
          ) : (
            <span />
          )}
          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={handleDiscard}
              disabled={!hasUnsavedChanges || saving}
            >
              {t('settings.general.save.discard')}
            </Button>
            <Button
              onClick={handleSave}
              disabled={!hasUnsavedChanges || saving || !!branchPrefixError}
            >
              {saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t('settings.general.save.button')}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
