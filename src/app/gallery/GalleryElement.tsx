import { ImageData } from '@/app/gallery/types.ts';
import { ReactElement, useEffect, useState } from 'react';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { DIRECTORY } from '@/app/gallery/GalleryPanel.tsx';

async function loadImageUrl(fileName: string) {
  const path = await invoke<string>('get_image_path', { directory: DIRECTORY, fileName });
  return convertFileSrc(path); // converts to file:// url usable in <img>
}

export type GalleryElementProps = {
  imageData: ImageData;
};

export function GalleryElement({ imageData }: GalleryElementProps): ReactElement {
  const [imageSource, setImageSource] = useState<string | null>(null);

  useEffect(() => {
    loadImageUrl(imageData.name).then((value) => setImageSource(value));
  }, [imageData.name]);

  if (imageSource == null) {
    return <></>;
  }

  return (
    <img
      src={imageSource}
      alt={'preview'}
      loading="lazy"
      style={{
        width: '100%',
        height: 'auto',
        objectFit: 'cover',
        borderRadius: '4px',
      }}
    />
  );
}
