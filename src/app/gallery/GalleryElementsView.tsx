import { MasonryView } from './views/MasonryView';
import { useGallery } from '@/app/gallery/galleryContext/useGallery.ts';
import { GalleryElement } from '@/app/gallery/elements/GalleryElement.tsx';

export function GalleryElementsView() {
  const { medias, isCurrentlyLoading, loadNextBatch } = useGallery();

  return (
    <MasonryView
      items={medias}
      getSize={(img) => ({ width: img.width, height: img.height })}
      renderItem={(img) => <GalleryElement imageElementData={img} />}
      isCurrentlyLoading={isCurrentlyLoading}
      loadNextBatch={loadNextBatch}
    />
  );
}
