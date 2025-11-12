import { NavLink, Outlet, useNavigate, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Settings, Cpu, Server, X, FolderOpen } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { useEffect } from 'react';
import { useHotkeysContext } from 'react-hotkeys-hook';
import { useKeyExit } from '@/keyboard/hooks';
import { Scope } from '@/keyboard/registry';

const settingsNavigation = [
  {
    path: 'general',
    icon: Settings,
  },
  {
    path: 'projects',
    icon: FolderOpen,
  },
  {
    path: 'agents',
    icon: Cpu,
  },
  {
    path: 'mcp',
    icon: Server,
  },
];

export function SettingsLayout() {
  const { t } = useTranslation('settings');
  const { enableScope, disableScope } = useHotkeysContext();

  // Enable SETTINGS scope when component mounts
  useEffect(() => {
    enableScope(Scope.SETTINGS);
    return () => {
      disableScope(Scope.SETTINGS);
    };
  }, [enableScope, disableScope]);

  const navigate = useNavigate();
  const location = useLocation();

  const handleBack = () => {
    if (location.state?.from) {
      navigate(-1);
    } else {
      navigate('/projects');
    }
  };
  // Register ESC keyboard shortcut
  useKeyExit(handleBack, { scope: Scope.SETTINGS });

  return (
    <div className="h-full overflow-auto">
      <div className="container mx-auto px-4 py-8">
        {/* Header with title and close button */}
        <div className="flex items-center justify-between sticky top-0 bg-background z-10 py-4 -mx-4 px-4">
          <h1 className="text-2xl font-semibold">
            {t('settings.layout.nav.title')}
          </h1>
          <Button
            variant="outline"
            size="sm"
            onClick={handleBack}
            className="h-8 px-3 flex items-center gap-2"
          >
            <X className="h-4 w-4" />
            <span className="text-xs">ESC</span>
          </Button>
        </div>
        <div className="flex flex-col lg:flex-row gap-8">
          {/* Sidebar Navigation */}
          <aside className="w-full lg:w-64 lg:shrink-0 lg:sticky lg:top-24 lg:h-fit lg:max-h-[calc(100vh-8rem)] lg:overflow-y-auto">
            <div className="space-y-1">
              <nav className="space-y-1">
                {settingsNavigation.map((item) => {
                  const Icon = item.icon;
                  return (
                    <NavLink
                      key={item.path}
                      to={item.path}
                      end
                      className={({ isActive }) =>
                        cn(
                          'flex items-start gap-3 px-3 py-2 text-sm rounded-md',
                          'hover:bg-accent/50',
                          isActive
                            ? 'bg-accent text-foreground'
                            : 'text-muted-foreground'
                        )
                      }
                    >
                      <Icon className="h-4 w-4 mt-0.5 shrink-0" />
                      <div className="flex-1 min-w-0">
                        <div className="font-medium">
                          {t(`settings.layout.nav.${item.path}`)}
                        </div>
                        <div className="text-xs opacity-70 mt-0.5">
                          {t(`settings.layout.nav.${item.path}Desc`)}
                        </div>
                      </div>
                    </NavLink>
                  );
                })}
              </nav>
            </div>
          </aside>

          {/* Main Content */}
          <main className="flex-1 min-w-0">
            <Outlet />
          </main>
        </div>
      </div>
    </div>
  );
}
