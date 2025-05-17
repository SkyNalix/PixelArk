import { useContext } from 'react';
import { MediaViewerContext } from './MediaViewerContext.ts';

export function useMediaViewer() {
  const context = useContext(MediaViewerContext);
  if (!context) {
    throw new Error('useMediaViewer must be used within a MediaViewerProvider');
  }
  return context;
}
