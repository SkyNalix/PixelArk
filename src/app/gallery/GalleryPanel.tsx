import { GalleryHeader } from '@/app/gallery/GalleryHeader.tsx';
import { Gallery } from '@/app/gallery/Gallery.tsx';

export function GalleryPanel() {
  return (
    <div
      style={{
        height: '100%',
        width: '100%',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      <div
        style={{
          position: 'sticky',
        }}
      >
        <GalleryHeader />
      </div>
      <Gallery />
    </div>
  );
}
