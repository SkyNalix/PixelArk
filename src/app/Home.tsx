import { AppSidebar } from '@/app/AppSideBar.tsx';
import { SidebarProvider } from '@/components/ui/sidebar';
import { GalleryPanel } from '@/app/gallery/GalleryPanel.tsx';

function Home() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <div
        style={{
          height: '100%',
          width: '100%',
          padding: 4,
          borderRadius: 'var(--radius)',
        }}
      >
        <GalleryPanel />
      </div>
    </SidebarProvider>
  );
}

export default Home;
