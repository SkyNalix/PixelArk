import { GalleryHeader } from '@/app/gallery/GalleryHeader.tsx';
import { GalleryProvider } from '@/app/gallery/galleryContext/GalleryProvider.tsx';
import { GalleryElementsView } from '@/app/gallery/GalleryElementsView.tsx';
import { GalleryFoldersView } from '@/app/gallery/GalleryFoldersView.tsx';
import { MediaViewerProvider } from '@/app/gallery/mediaViewer/mediaViewerContext/MediaViewerProvider.tsx';
import { useGallery } from '@/app/gallery/galleryContext/useGallery.ts';
import { MediaViewerPanel } from '@/app/gallery/mediaViewer/MediaViewerPanel.tsx';
import { useMediaViewer } from '@/app/gallery/mediaViewer/mediaViewerContext/useMediaViewer.ts';

function GalleryPanelInternal() {
  const { currentMedia } = useMediaViewer();

  return (
    <>
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
      {currentMedia && <MediaViewerPanel />}
    </>
  );
}

function GalleryPanelWithProvider() {
  const { medias } = useGallery();
  return (
    <MediaViewerProvider medias={medias}>
      <GalleryPanelInternal />
    </MediaViewerProvider>
  );
}

export function GalleryPanel() {
  return (
    <GalleryProvider>
      <GalleryPanelWithProvider />
    </GalleryProvider>
  );
}
