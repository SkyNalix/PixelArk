import { useMediaViewer } from '@/app/gallery/mediaViewer/mediaViewerContext/useMediaViewer.ts';
import { useCallback, useEffect } from 'react';

export function MediaViewerPanel() {
  const { currentMedia, setCurrentMedia, nextMedia, previousMedia } = useMediaViewer();

  const onClose = useCallback(() => {
    setCurrentMedia(null);
  }, [setCurrentMedia]);

  useEffect(() => {
    // Disable scroll
    document.body.style.overflow = 'hidden';

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      else if (e.key === 'ArrowRight') nextMedia();
      else if (e.key === 'ArrowLeft') previousMedia();
    };
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      // Re-enable scroll when unmounted
      document.body.style.overflow = 'auto';
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [nextMedia, onClose, previousMedia]);

  if (!currentMedia) {
    return <></>;
  }

  return (
    <div
      style={{
        position: 'fixed',
        height: '100%',
        width: '100%',
        top: 0,
        left: 0,
        zIndex: 1000,
        backgroundColor: 'var(--base)',
      }}
    >
      <div
        style={{
          height: '100%',
          width: '100%',
          display: 'flex',
          justifyContent: 'center',
        }}
      >
        <img
          src={currentMedia.path}
          alt={currentMedia.name}
          loading="lazy"
          style={{ width: 'max-content', height: '100%' }}
        />
      </div>
    </div>
  );
}
