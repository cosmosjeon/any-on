import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarSeparator,
} from '@/components/ui/sidebar';
import { useSidebar } from '@/components/ui/sidebar';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { Button } from '@/components/ui/button';
import {
  PanelLeftOpen,
  PanelLeftClose,
  FileText,
  Palette,
  KanbanSquare,
  FolderOpen,
  Settings,
  Plus,
  BookOpen,
  MessageCircleQuestion
} from 'lucide-react';
import AnyonLogo from '@/../../assets/logo/anyon.svg';
import AnyonLetterLogo from '@/../../assets/logo/ANYON-letter.svg';
import { Link, useLocation } from 'react-router-dom';
import { useProject } from '@/contexts/project-context';
import { openTaskForm } from '@/lib/openTaskForm';

export function AppSidebar() {
  const { state, toggleSidebar } = useSidebar();
  const isCollapsed = state === 'collapsed';
  const location = useLocation();
  const { projectId } = useProject();

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader className="border-b p-0">
        <div className="flex items-center justify-between h-12 w-full px-2">
          {isCollapsed ? (
            <TooltipProvider delayDuration={0}>
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    onClick={toggleSidebar}
                    className="flex items-center justify-center hover:bg-accent rounded-md transition-colors group p-1.5 mx-auto"
                  >
                    <img
                      src={AnyonLogo}
                      alt="Anyon Logo"
                      className="w-6 h-6 flex-shrink-0 group-hover:hidden"
                    />
                    <PanelLeftOpen className="w-5 h-5 flex-shrink-0 hidden group-hover:block" />
                  </button>
                </TooltipTrigger>
                <TooltipContent side="right">
                  <p>Open sidebar</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          ) : (
            <>
              <Link to="/projects" className="flex items-center gap-2 hover:opacity-80 transition-opacity">
                <img
                  src={AnyonLogo}
                  alt="Anyon Logo"
                  className="w-6 h-6 flex-shrink-0"
                />
                <img
                  src={AnyonLetterLogo}
                  alt="ANYON"
                  className="h-4 flex-shrink-0"
                />
              </Link>
              <button
                onClick={toggleSidebar}
                className="flex items-center justify-center hover:bg-accent rounded-md transition-colors p-1.5"
              >
                <PanelLeftClose className="w-5 h-5" />
              </button>
            </>
          )}
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarMenu>
            {projectId ? (
              // Project context: Show Docs, Design, Kanban
              <>
                <SidebarMenuItem>
                  <SidebarMenuButton asChild tooltip="Docs" isActive={location.pathname.includes('/docs')}>
                    <Link to={`/projects/${projectId}/docs`}>
                      <FileText className="w-5 h-5" />
                      <span>Docs</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
                <SidebarMenuItem>
                  <SidebarMenuButton asChild tooltip="Design" isActive={location.pathname.includes('/design')}>
                    <Link to={`/projects/${projectId}/design`}>
                      <Palette className="w-5 h-5" />
                      <span>Design</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
                <SidebarMenuItem>
                  <SidebarMenuButton asChild tooltip="Kanban" isActive={location.pathname.includes('/kanban')}>
                    <Link to={`/projects/${projectId}/kanban`}>
                      <KanbanSquare className="w-5 h-5" />
                      <span>Kanban</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </>
            ) : (
              // No project context: Show Projects
              <>
                <SidebarMenuItem>
                  <SidebarMenuButton asChild tooltip="Projects" isActive={location.pathname === '/projects' || location.pathname === '/'}>
                    <Link to="/projects">
                      <FolderOpen className="w-5 h-5" />
                      <span>Projects</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </>
            )}
          </SidebarMenu>
        </SidebarGroup>

        {/* Settings and External Links */}
        <SidebarGroup>
          <SidebarSeparator />
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton asChild tooltip="Settings" isActive={location.pathname.startsWith('/settings')}>
                <Link to={projectId ? `/settings/projects?projectId=${projectId}` : '/settings/general'}>
                  <Settings className="w-5 h-5" />
                  <span>Settings</span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
            <SidebarMenuItem>
              <SidebarMenuButton asChild tooltip="Documentation">
                <a href="https://anyon.com/docs" target="_blank" rel="noopener noreferrer">
                  <BookOpen className="w-5 h-5" />
                  <span>Docs</span>
                </a>
              </SidebarMenuButton>
            </SidebarMenuItem>
            <SidebarMenuItem>
              <SidebarMenuButton asChild tooltip="Support">
                <a href="https://github.com/slit/anyon/issues" target="_blank" rel="noopener noreferrer">
                  <MessageCircleQuestion className="w-5 h-5" />
                  <span>Support</span>
                </a>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="border-t">
        {projectId && (
          <div className="p-2">
            <TooltipProvider delayDuration={0}>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    onClick={() => openTaskForm({ projectId })}
                    className="w-full"
                    size={isCollapsed ? 'icon' : 'default'}
                  >
                    <Plus className="h-4 w-4" />
                    {!isCollapsed && <span className="ml-2">Create Task</span>}
                  </Button>
                </TooltipTrigger>
                {isCollapsed && (
                  <TooltipContent side="right">
                    <p>Create new task</p>
                  </TooltipContent>
                )}
              </Tooltip>
            </TooltipProvider>
          </div>
        )}
      </SidebarFooter>
    </Sidebar>
  );
}
