import { ReactNode, useCallback, useState } from 'react';
import { ImageElementData } from '@/app/gallery/types.ts';
import { MediaViewerContext } from './MediaViewerContext.ts';

export interface MediaViewerContextType {
  currentMedia: ImageElementData | null;

  setCurrentMedia: (media: ImageElementData | null) => void;
  nextMedia: () => void;
  previousMedia: () => void;
}

export function MediaViewerProvider({ children, medias }: { children: ReactNode; medias: ImageElementData[] }) {
  const [currentMedia, setCurrentMedia] = useState<ImageElementData | null>(null);

  const nextMedia = useCallback(() => {
    setCurrentMedia((prev) => {
      if (prev == null || prev.index == medias.length - 1) return prev;
      return prev.index < medias.length - 1 ? medias[prev.index + 1] : null;
    });
  }, [medias, setCurrentMedia]);

  const previousMedia = useCallback(() => {
    setCurrentMedia((prev) => {
      if (prev == null || prev.index <= 0) return prev;
      return prev.index > 0 ? medias[prev.index - 1] : null;
    });
  }, [medias, setCurrentMedia]);

  const value: MediaViewerContextType = {
    currentMedia,

    setCurrentMedia,
    nextMedia,
    previousMedia,
  };

  return <MediaViewerContext.Provider value={value}>{children}</MediaViewerContext.Provider>;
}
