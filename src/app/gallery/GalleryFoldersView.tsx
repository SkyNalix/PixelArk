import { GridView } from '@/app/gallery/views/GridView.tsx';
import { GalleryFolder } from '@/app/gallery/elements/GalleryFolder.tsx';
import { useCallback, useRef } from 'react';
import { useGallery } from '@/app/gallery/context/useGallery.ts';

export function GalleryFoldersView() {
  const { folderNames, setCurrentDirectory } = useGallery();

  const isClickLocked = useRef(false);

  const handleClick = useCallback(
    (folderName: string) => {
      if (isClickLocked.current) return;

      isClickLocked.current = true;
      setCurrentDirectory((currentDirectory) => [...currentDirectory, folderName]);

      setTimeout(() => {
        isClickLocked.current = false;
      }, 300); // Adjust timing as needed
    },
    [setCurrentDirectory],
  );

  return (
    <GridView
      items={folderNames}
      renderItem={(folderName) => <GalleryFolder folderName={folderName} onClick={handleClick} />}
    />
  );
}
