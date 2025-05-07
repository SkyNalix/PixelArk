import { createContext } from 'react';
import { GalleryContextType } from '@/app/gallery/context/GalleryProvider.tsx';

export const GalleryContext = createContext<GalleryContextType | null>(null);
