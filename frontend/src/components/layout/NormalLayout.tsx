import { Outlet } from 'react-router-dom';
import { SidebarProvider, SidebarInset } from '@/components/ui/sidebar';
import { AppSidebar } from '@/components/layout/app-sidebar';

export function NormalLayout() {
  return (
    <SidebarProvider defaultOpen={true}>
      <AppSidebar />
      <SidebarInset className="flex flex-col flex-1">
        <div className="flex-1 min-h-0 overflow-hidden">
          <Outlet />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
