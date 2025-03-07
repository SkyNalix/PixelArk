import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar';
import { Image, Settings } from 'lucide-react';

export function AppSidebar() {
  return (
    <Sidebar variant="floating">
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Application</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem key={'gallery'}>
                <SidebarMenuButton
                  onClick={() => {
                    console.log('switching panel to the gallery');
                  }}
                >
                  <Image />
                  <span>{'Gallery'}</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem key={'settings'}>
                <SidebarMenuButton
                  onClick={() => {
                    console.log('switching panel to the settings');
                  }}
                >
                  <Settings />
                  <span>{'Settings'}</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
