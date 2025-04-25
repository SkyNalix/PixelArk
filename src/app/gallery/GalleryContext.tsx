import React, { createContext, ReactNode, useCallback, useContext, useEffect, useRef, useState } from 'react';
import { ImageData } from './types';
import { invoke } from '@tauri-apps/api/core';

const BATCH_SIZE = 30;

interface GalleryContextType {
  // States
  medias: ImageData[];
  isCurrentlyLoading: boolean;
  totalImagesCount: number | null;
  folderNames: string[];
  currentDirectory: string[];

  // Methods
  loadNextBatch: () => void;
  isBatchLoaded: (start: number, end: number) => boolean;
  setMedias: React.Dispatch<React.SetStateAction<ImageData[]>>;
  setIsCurrentlyLoading: React.Dispatch<React.SetStateAction<boolean>>;
  setTotalImagesCount: React.Dispatch<React.SetStateAction<number | null>>;
  setCurrentDirectory: React.Dispatch<React.SetStateAction<string[]>>;
}

const GalleryContext = createContext<GalleryContextType | null>(null);

interface GalleryProviderProps {
  children: ReactNode;
}

export function GalleryProvider({ children }: GalleryProviderProps) {
  const currentBatch = useRef(0);
  const lastBatchLoadCall = useRef<number | null>(null);

  const [currentDirectory, setCurrentDirectory] = useState<string[]>([]);
  const [medias, setMedias] = useState<ImageData[]>([]);
  const [isCurrentlyLoading, setIsCurrentlyLoading] = useState(false);
  const [totalImagesCount, setTotalImagesCount] = useState<number | null>(null);
  const [folderNames, setFolderNames] = useState<string[]>([]);

  useEffect(() => {
    invoke<string[]>('get_folder_names', { directory: currentDirectory.join('/') })
      .then((names) => setFolderNames(names))
      .catch((error) => {
        console.error('Failed to get folder names:', error);
        setFolderNames([]);
      });
  }, [currentDirectory]);

  // Function to load a specific batch of images
  const loadBatchImages = useCallback(
    async (start: number, stop: number) => {
      setIsCurrentlyLoading(true);

      try {
        const newImages = await invoke<ImageData[]>('load_images_from_directory', {
          directory: currentDirectory.join('/'),
          start,
          stop,
        });

        if (newImages.length === 0) return;

        setMedias((prev) => [...prev, ...newImages]);
        currentBatch.current += 1;
      } catch (error) {
        console.error(error);
      } finally {
        setIsCurrentlyLoading(false);
      }
    },
    [currentDirectory],
  );

  // Check if batch is already loaded
  const isBatchLoaded = useCallback(
    (start: number, end: number): boolean => {
      return start < medias.length && end <= medias.length;
    },
    [medias],
  );

  // Load next batch of images when scrolling down
  const loadNextBatch = useCallback(() => {
    if (isCurrentlyLoading || lastBatchLoadCall.current == currentBatch.current) return;

    const startIndex = currentBatch.current * BATCH_SIZE;
    const endIndex = startIndex + BATCH_SIZE;

    if (isBatchLoaded(startIndex, endIndex)) return;
    lastBatchLoadCall.current = currentBatch.current;
    loadBatchImages(startIndex, endIndex);
  }, [currentBatch, isCurrentlyLoading, loadBatchImages, isBatchLoaded]);

  // on current directory change
  useEffect(() => {
    setMedias([]);
    setIsCurrentlyLoading(false);
    setTotalImagesCount(null);
    currentBatch.current = 0;
  }, [currentDirectory]);

  const value = {
    // States
    medias,
    isCurrentlyLoading,
    totalImagesCount,
    folderNames,
    currentDirectory,
    // Methods
    loadNextBatch,
    isBatchLoaded,
    setMedias,
    setIsCurrentlyLoading,
    setTotalImagesCount,
    setCurrentDirectory,
  };

  return <GalleryContext.Provider value={value}>{children}</GalleryContext.Provider>;
}

export function useGallery() {
  const context = useContext(GalleryContext);
  if (!context) {
    throw new Error('useGallery must be used within a GalleryProvider');
  }
  return context;
}
