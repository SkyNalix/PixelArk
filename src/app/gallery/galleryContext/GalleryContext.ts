import { createContext } from 'react';
import { GalleryContextType } from '@/app/gallery/galleryContext/GalleryProvider.tsx';

export const GalleryContext = createContext<GalleryContextType | null>(null);
