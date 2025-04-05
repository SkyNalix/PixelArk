import React, { createContext, ReactNode, useCallback, useContext, useEffect, useState } from 'react';
import { ImageData } from './types';
import { invoke } from '@tauri-apps/api/core';

const BATCH_SIZE = 30;

interface GalleryContextType {
  // States
  medias: ImageData[];
  isCurrentlyLoading: boolean;
  totalImagesCount: number | null;
  currentBatch: number;
  folderNames: string[];
  currentDirectory: string[];

  // Methods
  loadBatchImages: (start: number, end: number) => Promise<void>;
  loadNextBatch: () => void;
  isBatchLoaded: (start: number, end: number) => boolean;
  setMedias: React.Dispatch<React.SetStateAction<ImageData[]>>;
  setIsCurrentlyLoading: React.Dispatch<React.SetStateAction<boolean>>;
  setTotalImagesCount: React.Dispatch<React.SetStateAction<number | null>>;
  setCurrentBatch: React.Dispatch<React.SetStateAction<number>>;
  setCurrentDirectory: React.Dispatch<React.SetStateAction<string[]>>;
}

const GalleryContext = createContext<GalleryContextType | null>(null);

interface GalleryProviderProps {
  rootPath: string;
  children: ReactNode;
}

export function GalleryProvider({ rootPath, children }: GalleryProviderProps) {
  const [currentDirectory, setCurrentDirectory] = useState<string[]>([]);
  const [medias, setMedias] = useState<ImageData[]>([]);
  const [isCurrentlyLoading, setIsCurrentlyLoading] = useState(false);
  const [totalImagesCount, setTotalImagesCount] = useState<number | null>(null);
  const [currentBatch, setCurrentBatch] = useState(0);
  const [folderNames, setFolderNames] = useState<string[]>([]);

  useEffect(() => {
    invoke<string[]>('get_folder_names', { directory: `${rootPath}/${currentDirectory.join('/')}` })
      .then((names) => setFolderNames(names))
      .catch((error) => {
        console.error('Failed to get folder names:', error);
        setFolderNames([]);
      });
  }, [currentDirectory, rootPath]);

  // Function to load a specific batch of images
  const loadBatchImages = useCallback(
    async (start: number, stop: number) => {
      setIsCurrentlyLoading(true);

      try {
        const newImages = await invoke<ImageData[]>('load_images_from_directory', {
          directory: `${rootPath}/${currentDirectory.join('/')}`,
          start,
          stop,
        });

        if (newImages.length === 0) return;

        const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });
        newImages.sort((a, b) => collator.compare(a.image_name, b.image_name));

        setMedias((prev) => [...prev, ...newImages]);
        setCurrentBatch((prev) => prev + 1);
      } catch (error) {
        console.error(error);
      } finally {
        setIsCurrentlyLoading(false);
      }
    },
    [currentDirectory, rootPath],
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
    if (isCurrentlyLoading) return;

    const startIndex = currentBatch * BATCH_SIZE;
    const endIndex = startIndex + BATCH_SIZE;

    if (isBatchLoaded(startIndex, endIndex)) return;

    loadBatchImages(startIndex, endIndex);
  }, [currentBatch, isCurrentlyLoading, loadBatchImages, isBatchLoaded]);

  const value = {
    // States
    medias,
    isCurrentlyLoading,
    totalImagesCount,
    currentBatch,
    folderNames,
    currentDirectory,
    // Methods
    loadBatchImages,
    loadNextBatch,
    isBatchLoaded,
    setMedias,
    setIsCurrentlyLoading,
    setTotalImagesCount,
    setCurrentBatch,
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
