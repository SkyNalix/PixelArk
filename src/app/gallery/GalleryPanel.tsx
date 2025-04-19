import { GalleryHeader } from '@/app/gallery/GalleryHeader.tsx';
import { MasonryView } from '@/app/gallery/views/MasonryView';
import { GalleryProvider } from './GalleryContext';
import { GridView } from './views/GridView';

const DIRECTORY = 'C:\\dev\\PixelArk\\images100';

export function GalleryPanel() {
  return (
    <GalleryProvider rootPath={DIRECTORY}>
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
          <GridView />
          <MasonryView />
        </div>
      </div>
    </GalleryProvider>
  );
}
