import { ReactElement, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { ImageData } from '@/app/gallery/types.ts';
import { useInView } from 'react-intersection-observer';
import { GalleryElement } from '../GalleryElement';
import { useGallery } from '@/app/gallery/GalleryContext.tsx';

const COLUMNS = 3;
const COLUMNS_GAP = 8;

type ColumnImageType = { imageData: ImageData; index: number };

export function MasonryView(): ReactElement {
  const { medias, isCurrentlyLoading, currentDirectory, loadNextBatch } = useGallery();

  const containerRef = useRef<HTMLDivElement | null>(null);
  const [containerWidth, setContainerWidth] = useState<number | null>(null);

  const columnWidth = containerWidth ? (containerWidth - COLUMNS_GAP * (COLUMNS - 1)) / COLUMNS : null;

  useLayoutEffect(() => {
    if (containerRef.current) {
      const resizeObserver = new ResizeObserver(([entry]) => {
        setContainerWidth(entry.contentRect.width);
      });

      resizeObserver.observe(containerRef.current);

      return () => resizeObserver.disconnect();
    }
  }, []);

  // Create columns for horizontal layout
  const columns = useMemo(() => {
    if (!columnWidth) {
      // Just dump all images into the first column
      const fallback: Array<Array<ColumnImageType>> = Array(COLUMNS)
        .fill([])
        .map(() => []);
      medias.forEach((imageData, index) => {
        fallback[0].push({ imageData, index });
      });
      return fallback;
    }

    const cols: Array<Array<ColumnImageType>> = Array(COLUMNS)
      .fill([])
      .map(() => []);
    const columnsSizes = Array(COLUMNS).fill(0);

    medias.forEach((imageData, index) => {
      const scale = columnWidth / imageData.width;
      const scaledHeight = imageData.height * scale;

      const columnIndex = columnsSizes.reduce(
        (shortestIndex, current, i, sizes) => (current < sizes[shortestIndex] ? i : shortestIndex),
        0,
      );

      columnsSizes[columnIndex] += scaledHeight;
      cols[columnIndex].push({ imageData, index });
    });

    return cols;
  }, [medias, columnWidth]);

  // intersection observer for bottom detection to load next batches
  const { ref: bottomObserverRef, inView: bottomInView } = useInView({
    threshold: 0.1,
    rootMargin: '200px 0px',
  });

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
        ref={containerRef}
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
              width: columnWidth ? `${columnWidth}px` : 'auto', // <- fixed width
            }}
          >
            {columnItems.map(({ imageData, index }) => (
              <GalleryElement key={index} imageData={imageData} />
            ))}
          </div>
        ))}
      </div>

      <div ref={bottomObserverRef}>{isCurrentlyLoading && <div>Loading...</div>}</div>
    </div>
  );
}
