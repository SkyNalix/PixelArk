import { ImageData } from '@/app/gallery/types.ts';
import { ReactElement } from 'react';

export type GalleryElementProps = {
  index: number;
  data: ImageData;
};

export function GalleryElement({ index, data: imageData }: GalleryElementProps): ReactElement {
  return (
    <div className="masonry-item" data-index={index}>
      <img
        src={imageData.image_base64}
        alt={imageData.image_name}
        style={{
          width: '100%',
          height: 'auto',
          objectFit: 'cover',
          borderRadius: '4px',
        }}
        loading="lazy"
      />
    </div>
  );
}
