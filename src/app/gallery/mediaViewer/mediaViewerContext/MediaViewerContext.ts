import { createContext } from 'react';
import { MediaViewerContextType } from '@/app/gallery/mediaViewer/mediaViewerContext/MediaViewerProvider.tsx';

export const MediaViewerContext = createContext<MediaViewerContextType | null>(null);
