import { MasonryView } from './views/MasonryView';
import { useGallery } from '@/app/gallery/GalleryContext.tsx';
import { GalleryElement } from '@/app/gallery/elements/GalleryElement.tsx';

export function GalleryElementsView() {
  const { medias, isCurrentlyLoading, loadNextBatch } = useGallery();

  return (
    <MasonryView
      items={medias}
      getSize={(img) => ({ width: img.width, height: img.height })}
      renderItem={(img) => <GalleryElement imageData={img} />}
      isCurrentlyLoading={isCurrentlyLoading}
      loadNextBatch={loadNextBatch}
    />
  );
}
