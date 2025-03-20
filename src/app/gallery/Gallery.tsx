import { ReactElement, useCallback, useEffect, useState, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ImageData } from '@/app/gallery/types.ts';
import { useInView } from 'react-intersection-observer';
import { GalleryElement } from './GalleryElement';

const SORT_COMPARATOR = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });

async function loadImages(directory: string, start: number, stop: number): Promise<ImageData[]> {
  return await invoke('load_images_from_directory', {
    directory,
    start,
    stop,
  });
}

const BATCH_SIZE = 30; // Multiple of 3 for even column distribution
const COLUMNS = 3;
const directory = 'C:\\dev\\PixelArk\\images10000';

export function Gallery(): ReactElement {
  // Stores all image data and positions in the gallery, can contain null for unloaded images
  const [medias, setMediaData] = useState<ImageData[]>([]);

  // Indicates whether new images are currently being loaded
  const [isCurrentlyLoading, setCurrentlyLoading] = useState(false);

  // Current batch number being displayed, used for pagination
  const [currentBatch, setCurrentBatch] = useState(0);

  // Maps each image index to its assigned column number for masonry layout
  const [columnAssignments, setColumnAssignments] = useState<number[]>([]);

  // Calculate column assignments when images change
  useEffect(() => {
    const newAssignments: number[] = [];
    const columnHeights = new Array(COLUMNS).fill(0);

    medias.forEach((imageData, index) => {
      // Find the shortest column
      const shortestColumnIndex = columnHeights.reduce((shortestIndex, current, currentIndex) => {
        return current < columnHeights[shortestIndex] ? currentIndex : shortestIndex;
      }, 0);

      // Assign image to shortest column
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
        const shortestColumnIndex = columnsSizes.reduce((shortestIndex, current, currentIndex) => {
          return current < columnsSizes[shortestIndex] ? currentIndex : shortestIndex;
        }, 0);
        columnIndex = shortestColumnIndex;
      }
      columnsSizes[columnIndex] += imageData.height;

      cols[columnIndex].push({ imageData, index });
    });

    return cols;
  }, [medias, columnAssignments]);

  // Setup intersection observer for bottom detection to load more images
  const { ref: bottomObserverRef, inView: bottomInView } = useInView({
    threshold: 0.1,
    rootMargin: '200px 0px',
  });

  // Function to load a specific batch of images
  const loadBatchImages = useCallback((start: number, end: number) => {
    setCurrentlyLoading(true);

    loadImages(directory, start, end)
      .then((newImages) => {
        if (newImages.length === 0) {
          setCurrentlyLoading(false);
          return;
        }

        newImages.sort((a, b) => SORT_COMPARATOR.compare(a.image_name, b.image_name));
        setMediaData((prev) => [...prev, ...newImages]);
      })
      .catch((error) => {
        console.error(error);
      })
      .finally(() => {
        setCurrentlyLoading(false);
      });
  }, []);

  // Check if batch is already loaded
  const isBatchLoaded = useCallback(
    (start: number, end: number): boolean => {
      return start < medias.length && end <= medias.length;
    },
    [medias],
  );

  // Load next batch of images when scrolling down
  const loadNextBatch = useCallback(() => {
    if (isCurrentlyLoading) return;

    const startIndex = currentBatch * BATCH_SIZE;
    const endIndex = startIndex + BATCH_SIZE;

    if (isBatchLoaded(startIndex, endIndex)) {
      setCurrentBatch((prev) => prev + 1);
      return;
    }

    loadBatchImages(startIndex, endIndex);
    setCurrentBatch((prev) => prev + 1);
  }, [currentBatch, isCurrentlyLoading, loadBatchImages, isBatchLoaded]);

  // Load next batch when bottom observer triggers
  useEffect(() => {
    if (bottomInView && !isCurrentlyLoading) {
      loadNextBatch();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bottomInView]);

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
            key={`column-${columnIndex}`}
            style={{
              display: 'flex',
              flexDirection: 'column',
              gap: '8px',
            }}
          >
            {columnItems.map(({ imageData, index }) => (
              <GalleryElement key={`pos-${index}`} index={index} data={imageData} />
            ))}
          </div>
        ))}
      </div>

      {/* Bottom observer to detect when to load the enxt batch */}
      <div ref={bottomObserverRef}>{isCurrentlyLoading && <div>Loading...</div>}</div>
    </div>
  );
}
