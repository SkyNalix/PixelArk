import { ReactElement, useCallback } from 'react';
import { GalleryFolder } from '../GalleryFolder';
import { useGallery } from '@/app/gallery/GalleryContext.tsx';

export function GridView(): ReactElement {
  const { folderNames, setCurrentDirectory } = useGallery();

  const onFolderClick = useCallback(
    (folderName: string) => {
      setCurrentDirectory((currentDirectory) => [...currentDirectory, folderName]);
    },
    [setCurrentDirectory],
  );

  return (
    <div
      style={{
        width: '100%',
        display: 'flex',
        flexDirection: 'row',
        flexWrap: 'wrap',
        gap: '8px',
      }}
    >
      {folderNames.map((folderName, index) => (
        <div key={index}>
          <GalleryFolder folderName={folderName} onClick={onFolderClick} />
        </div>
      ))}
    </div>
  );
}
