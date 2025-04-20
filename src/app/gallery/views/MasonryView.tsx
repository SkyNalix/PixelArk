import { ReactElement, useEffect, useMemo, useState } from 'react';
import { ImageData } from '@/app/gallery/types.ts';
import { useInView } from 'react-intersection-observer';
import { GalleryElement } from '../GalleryElement';
import { useGallery } from '@/app/gallery/GalleryContext.tsx';

const COLUMNS = 3;

export function MasonryView(): ReactElement {
  const { medias, isCurrentlyLoading, currentDirectory, loadNextBatch } = useGallery();

  const [columnAssignments, setColumnAssignments] = useState<number[]>([]);

  // Setup intersection observer for bottom detection to load more images
  const { ref: bottomObserverRef, inView: bottomInView } = useInView({
    threshold: 0.1,
    rootMargin: '200px 0px',
  });

  // Calculate column assignments when images change
  useEffect(() => {
    const newAssignments: number[] = [];
    const columnHeights = new Array(COLUMNS).fill(0);

    medias.forEach((imageData, index) => {
      // Find the shortest column
      const shortestColumnIndex = columnHeights.reduce((shortestIndex, current, currentIndex) => {
        return current < columnHeights[shortestIndex] ? currentIndex : shortestIndex;
      }, 0);

      // Assign image to the shortest column
      newAssignments[index] = shortestColumnIndex;

      // Update column height with estimated height
      if (imageData) {
        const aspectRatio = imageData.height / imageData.width;
        const estimatedHeight = aspectRatio * (100 / COLUMNS);
        columnHeights[shortestColumnIndex] += estimatedHeight;
      }
    });

    setColumnAssignments(newAssignments);
  }, [medias]);

  // Create columns for horizontal layout
  const columns = useMemo(() => {
    const cols = Array(COLUMNS)
      .fill(null)
      .map(() => [] as { imageData: ImageData; index: number }[]);

    const columnsSizes = Array(COLUMNS).fill(0);

    // Distribute images based on stored assignments
    medias.forEach((imageData, index) => {
      let columnIndex = columnAssignments[index];
      if (columnIndex === undefined) {
        columnIndex = columnsSizes.reduce((shortestIndex, current, currentIndex) => {
          return current < columnsSizes[shortestIndex] ? currentIndex : shortestIndex;
        }, 0);
      }
      columnsSizes[columnIndex] += imageData.height;

      cols[columnIndex].push({ imageData, index });
    });

    return cols;
  }, [medias, columnAssignments]);

  // Load next batch when bottom observer triggers
  useEffect(() => {
    if (bottomInView && !isCurrentlyLoading) {
      loadNextBatch();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bottomInView, currentDirectory]);

  return (
    <div>
      <div
        style={{
          display: 'flex',
          flexDirection: 'row',
          width: '100%',
          gap: '8px',
        }}
      >
        {columns.map((columnItems, columnIndex) => (
          <div
            key={columnIndex}
            style={{
              display: 'flex',
              flexDirection: 'column',
              flexGrow: 1,
              gap: '8px',
            }}
          >
            {columnItems.map(({ imageData, index }) => (
              <GalleryElement key={index} imageData={imageData} />
            ))}
          </div>
        ))}
      </div>

      {/* Bottom observer to detect when to load the next batch */}
      <div ref={bottomObserverRef}>{isCurrentlyLoading && <div>Loading...</div>}</div>
    </div>
  );
}
