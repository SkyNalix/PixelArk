import { ImageData } from '@/app/gallery/types.ts';
import { ReactElement, useEffect, useState } from 'react';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';

async function loadImageUrl(filePath: string) {
  const path = await invoke<string>('get_image_path', { filePath: filePath });
  return convertFileSrc(path); // converts to file:// url usable in <img>
}

export type GalleryElementProps = {
  imageData: ImageData;
};

export function GalleryElement({ imageData }: GalleryElementProps): ReactElement {
  const [imageSource, setImageSource] = useState<string | null>(null);

  useEffect(() => {
    loadImageUrl(imageData.path)
      .then((value) => setImageSource(value))
      .catch((error) => console.error(error));
  }, [imageData.path]);

  if (imageSource == null) {
    return <></>;
  }

  return (
    <img
      src={imageSource}
      alt={imageData.name}
      loading="lazy"
      style={{
        width: '100%',
        height: 'auto',
        borderRadius: '4px',
      }}
      onError={() => console.error('Broken image:', imageData.path)}
    />
  );
}
