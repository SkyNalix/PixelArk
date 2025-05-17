import { ImageElementData } from '@/app/gallery/types.ts';
import { ReactElement, useCallback } from 'react';
import { useMediaViewer } from '@/app/gallery/mediaViewer/mediaViewerContext/useMediaViewer.ts';

export type GalleryElementProps = {
  imageElementData: ImageElementData;
};

export function GalleryElement({ imageElementData }: GalleryElementProps): ReactElement {
  const { setCurrentMedia } = useMediaViewer();

  const clickAction = useCallback(() => {
    console.log('Clicked on image:', imageElementData.path);
    setCurrentMedia(imageElementData);
  }, [imageElementData, setCurrentMedia]);

  return (
    <>
      <img
        src={imageElementData.thumbnail_path}
        alt={imageElementData.name}
        loading="lazy"
        style={{
          width: '100%',
          height: 'auto',
          borderRadius: '4px',
        }}
        onError={() => console.error('Broken image:', imageElementData.thumbnail_path)}
        onClick={clickAction}
      />
    </>
  );
}
