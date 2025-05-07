import { ImageElementData } from '@/app/gallery/types.ts';
import { ReactElement } from 'react';

export type GalleryElementProps = {
  imageElementData: ImageElementData;
};

export function GalleryElement({ imageElementData }: GalleryElementProps): ReactElement {
  return (
    <img
      src={imageElementData.path}
      alt={imageElementData.name}
      loading="lazy"
      style={{
        width: '100%',
        height: 'auto',
        borderRadius: '4px',
      }}
      onError={() => console.error('Broken image:', imageElementData.path)}
    />
  );
}
