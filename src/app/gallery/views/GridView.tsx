import { ReactElement, useCallback, useRef } from 'react';
import { GalleryFolder } from '../GalleryFolder';
import { useGallery } from '@/app/gallery/GalleryContext.tsx';

export function GridView(): ReactElement {
  const { folderNames, setCurrentDirectory } = useGallery();

  const isClickLocked = useRef(false);

  const onFolderClick = useCallback(
    (folderName: string) => {
      if (isClickLocked.current) return;

      isClickLocked.current = true;
      setCurrentDirectory((currentDirectory) => [...currentDirectory, folderName]);

      setTimeout(() => {
        isClickLocked.current = false;
      }, 300); // adjust timing as needed
    },
    [setCurrentDirectory],
  );

  if (folderNames.length === 0) {
    return <></>;
  }

  return (
    <div
      style={{
        width: '100%',
        display: 'flex',
        flexDirection: 'row',
        flexWrap: 'wrap',
        gap: '8px',
        paddingBottom: '20px',
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
