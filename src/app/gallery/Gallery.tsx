import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function Gallery() {
  const [images, setImages] = useState<string[]>([]);

  useEffect(() => {
    async function fetchImages() {
      const result: string[] = await invoke('load_images_from_directory', {
        directory: 'C:\\Users\\SkyNalix\\Pictures',
      });
      setImages(result);
    }

    fetchImages().catch((e) => console.log(e));
  }, []);

  return (
    <div className="grid grid-cols-3 gap-4 p-4">
      {images.map((imgSrc, index) => (
        <img key={index} src={imgSrc} alt={`Gallery ${index}`} className="w-full h-auto rounded-lg shadow-lg" />
      ))}
    </div>
  );
}
