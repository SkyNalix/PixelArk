import { GalleryHeader } from '@/app/gallery/GalleryHeader.tsx';
import { GalleryProvider } from './context/GalleryProvider.tsx';
import { GalleryElementsView } from '@/app/gallery/GalleryElementsView.tsx';
import { GalleryFoldersView } from '@/app/gallery/GalleryFoldersView.tsx';

export function GalleryPanel() {
  return (
    <GalleryProvider>
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <div
          style={{
            position: 'sticky',
            top: 0,
          }}
        >
          <GalleryHeader />
        </div>
        <div
          style={{
            width: '100%',
            display: 'flex',
            flexDirection: 'column',
            gap: '8px',
            paddingTop: '8px',
          }}
        >
          <GalleryFoldersView />
          <GalleryElementsView />
        </div>
      </div>
    </GalleryProvider>
  );
}
