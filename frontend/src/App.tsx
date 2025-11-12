import { useEffect, lazy, Suspense } from 'react';
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { I18nextProvider } from 'react-i18next';
import i18n from '@/i18n';
import { NormalLayout } from '@/components/layout/NormalLayout';
import { usePostHog } from 'posthog-js/react';

// Lazy load page components for code splitting
const Projects = lazy(() => import('@/pages/projects').then(m => ({ default: m.Projects })));
const ProjectTasks = lazy(() => import('@/pages/project-tasks').then(m => ({ default: m.ProjectTasks })));
const FullAttemptLogsPage = lazy(() => import('@/pages/full-attempt-logs').then(m => ({ default: m.FullAttemptLogsPage })));
const SettingsLayout = lazy(() => import('@/pages/settings/').then(m => ({ default: m.SettingsLayout })));
const GeneralSettings = lazy(() => import('@/pages/settings/').then(m => ({ default: m.GeneralSettings })));
const ProjectSettings = lazy(() => import('@/pages/settings/').then(m => ({ default: m.ProjectSettings })));
const AgentSettings = lazy(() => import('@/pages/settings/').then(m => ({ default: m.AgentSettings })));
const McpSettings = lazy(() => import('@/pages/settings/').then(m => ({ default: m.McpSettings })));
import {
  UserSystemProvider,
  useUserSystem,
} from '@/components/config-provider';
import { ThemeProvider } from '@/components/theme-provider';
import { SearchProvider } from '@/contexts/search-context';

import { HotkeysProvider } from 'react-hotkeys-hook';

import { ProjectProvider } from '@/contexts/project-context';
import { ThemeMode } from 'shared/types';
import * as Sentry from '@sentry/react';
import { Loader } from '@/components/ui/loader';

import NiceModal from '@ebay/nice-modal-react';
import { OnboardingResult } from './components/dialogs/global/OnboardingDialog';
import { ClickedElementsProvider } from './contexts/ClickedElementsProvider';
import { PageErrorBoundary } from '@/components/ErrorBoundary';

const SentryRoutes = Sentry.withSentryReactRouterV6Routing(Routes);

function AppContent() {
  const { config, analyticsUserId, updateAndSaveConfig, loading } =
    useUserSystem();
  const posthog = usePostHog();

  // Handle opt-in/opt-out and user identification when config loads
  useEffect(() => {
    if (!posthog || !analyticsUserId) return;

    const userOptedIn = config?.analytics_enabled !== false;

    if (userOptedIn) {
      posthog.opt_in_capturing();
      posthog.identify(analyticsUserId);
      console.log('[Analytics] Analytics enabled and user identified');
    } else {
      posthog.opt_out_capturing();
      console.log('[Analytics] Analytics disabled by user preference');
    }
  }, [config?.analytics_enabled, analyticsUserId, posthog]);

  useEffect(() => {
    let cancelled = false;

    const handleOnboardingComplete = async (
      onboardingConfig: OnboardingResult
    ) => {
      if (cancelled) return;
      const updatedConfig = {
        ...config,
        onboarding_acknowledged: true,
        executor_profile: onboardingConfig.profile,
        editor: onboardingConfig.editor,
      };

      updateAndSaveConfig(updatedConfig);
    };

    const handleDisclaimerAccept = async () => {
      if (cancelled) return;
      await updateAndSaveConfig({ disclaimer_acknowledged: true });
    };

    const handleGitHubLoginComplete = async () => {
      if (cancelled) return;
      await updateAndSaveConfig({ github_login_acknowledged: true });
    };

    const handleTelemetryOptIn = async (analyticsEnabled: boolean) => {
      if (cancelled) return;
      await updateAndSaveConfig({
        telemetry_acknowledged: true,
        analytics_enabled: analyticsEnabled,
      });
    };

    const handleReleaseNotesClose = async () => {
      if (cancelled) return;
      await updateAndSaveConfig({ show_release_notes: false });
    };

    const checkOnboardingSteps = async () => {
      if (!config || cancelled) return;

      if (!config.disclaimer_acknowledged) {
        await NiceModal.show('disclaimer');
        await handleDisclaimerAccept();
        await NiceModal.hide('disclaimer');
      }

      if (!config.onboarding_acknowledged) {
        const onboardingResult: OnboardingResult =
          await NiceModal.show('onboarding');
        await handleOnboardingComplete(onboardingResult);
        await NiceModal.hide('onboarding');
      }

      if (!config.github_login_acknowledged) {
        await NiceModal.show('github-login');
        await handleGitHubLoginComplete();
        await NiceModal.hide('github-login');
      }

      if (!config.telemetry_acknowledged) {
        const analyticsEnabled: boolean =
          await NiceModal.show('privacy-opt-in');
        await handleTelemetryOptIn(analyticsEnabled);
        await NiceModal.hide('privacy-opt-in');
      }

      if (config.show_release_notes) {
        await NiceModal.show('release-notes');
        await handleReleaseNotesClose();
        await NiceModal.hide('release-notes');
      }
    };

    const runOnboarding = async () => {
      if (!config || cancelled) return;
      await checkOnboardingSteps();
    };

    runOnboarding();

    return () => {
      cancelled = true;
    };
  }, [config]);

  if (loading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <Loader message="Loading..." size={32} />
      </div>
    );
  }

  return (
    <I18nextProvider i18n={i18n}>
      <ThemeProvider initialTheme={config?.theme || ThemeMode.SYSTEM}>
        <SearchProvider>
          <div className="h-screen flex flex-col bg-background">
            <Suspense
              fallback={
                <div className="min-h-screen bg-background flex items-center justify-center">
                  <Loader message="Loading..." size={32} />
                </div>
              }
            >
              <SentryRoutes>
                {/* VS Code full-page logs route (outside NormalLayout for minimal UI) */}
                <Route
                  path="/projects/:projectId/tasks/:taskId/attempts/:attemptId/full"
                  element={
                    <PageErrorBoundary>
                      <FullAttemptLogsPage />
                    </PageErrorBoundary>
                  }
                />

                <Route element={<NormalLayout />}>
                  <Route
                    path="/"
                    element={
                      <PageErrorBoundary>
                        <Projects />
                      </PageErrorBoundary>
                    }
                  />
                  <Route
                    path="/projects"
                    element={
                      <PageErrorBoundary>
                        <Projects />
                      </PageErrorBoundary>
                    }
                  />
                  <Route
                    path="/projects/:projectId"
                    element={
                      <PageErrorBoundary>
                        <Projects />
                      </PageErrorBoundary>
                    }
                  />
                  <Route
                    path="/projects/:projectId/tasks"
                    element={
                      <PageErrorBoundary>
                        <ProjectTasks />
                      </PageErrorBoundary>
                    }
                  />
                  <Route path="/settings/*" element={<SettingsLayout />}>
                    <Route index element={<Navigate to="general" replace />} />
                    <Route
                      path="general"
                      element={
                        <PageErrorBoundary>
                          <GeneralSettings />
                        </PageErrorBoundary>
                      }
                    />
                    <Route
                      path="projects"
                      element={
                        <PageErrorBoundary>
                          <ProjectSettings />
                        </PageErrorBoundary>
                      }
                    />
                    <Route
                      path="agents"
                      element={
                        <PageErrorBoundary>
                          <AgentSettings />
                        </PageErrorBoundary>
                      }
                    />
                    <Route
                      path="mcp"
                      element={
                        <PageErrorBoundary>
                          <McpSettings />
                        </PageErrorBoundary>
                      }
                    />
                  </Route>
                  <Route
                    path="/mcp-servers"
                    element={<Navigate to="/settings/mcp" replace />}
                  />
                  <Route
                    path="/projects/:projectId/tasks/:taskId"
                    element={
                      <PageErrorBoundary>
                        <ProjectTasks />
                      </PageErrorBoundary>
                    }
                  />
                  <Route
                    path="/projects/:projectId/tasks/:taskId/attempts/:attemptId"
                    element={
                      <PageErrorBoundary>
                        <ProjectTasks />
                      </PageErrorBoundary>
                    }
                  />
                </Route>
              </SentryRoutes>
            </Suspense>
          </div>
        </SearchProvider>
      </ThemeProvider>
    </I18nextProvider>
  );
}

function App() {
  return (
    <BrowserRouter>
      <UserSystemProvider>
        <ClickedElementsProvider>
          <ProjectProvider>
            <HotkeysProvider initiallyActiveScopes={['*', 'global', 'kanban']}>
              <NiceModal.Provider>
                <AppContent />
              </NiceModal.Provider>
            </HotkeysProvider>
          </ProjectProvider>
        </ClickedElementsProvider>
      </UserSystemProvider>
    </BrowserRouter>
  );
}

export default App;
