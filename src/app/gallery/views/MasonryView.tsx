import { ReactElement, useLayoutEffect, useMemo, useRef, useState, useEffect } from 'react';
import { useInView } from 'react-intersection-observer';

const COLUMNS = 3;
const COLUMNS_GAP = 8;

type ColumnItemType<T> = {
  item: T;
  index: number;
};

type MasonryViewProps<T> = {
  items: T[];
  getSize: (item: T) => { width: number; height: number };
  renderItem: (item: T, index: number) => ReactElement;
  isCurrentlyLoading: boolean;
  loadNextBatch: () => void;
};

export function MasonryView<T>({
  items,
  getSize,
  renderItem,
  isCurrentlyLoading,
  loadNextBatch,
}: MasonryViewProps<T>): ReactElement {
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
      const fallback: Array<Array<ColumnItemType<T>>> = Array.from({ length: COLUMNS }, () => []);
      items.forEach((item, index) => {
        fallback[0].push({ item, index });
      });
      return fallback;
    }

    const cols: Array<Array<ColumnItemType<T>>> = Array.from({ length: COLUMNS }, () => []);
    const columnsHeights = Array(COLUMNS).fill(0);

    items.forEach((item, index) => {
      const size = getSize(item);
      const scale = columnWidth / size.width;
      const scaledHeight = size.height * scale;

      const columnIndex = columnsHeights.reduce(
        (minIndex, height, i, arr) => (height < arr[minIndex] ? i : minIndex),
        0,
      );

      columnsHeights[columnIndex] += scaledHeight;
      cols[columnIndex].push({ item, index });
    });

    return cols;
  }, [items, columnWidth, getSize]);

  // Intersection observer for loading next batch
  const { ref: bottomObserverRef, inView: bottomInView } = useInView({
    threshold: 0.1,
    rootMargin: '200px 0px',
  });

  useEffect(() => {
    if (bottomInView && !isCurrentlyLoading) {
      loadNextBatch();
    }
  }, [bottomInView, isCurrentlyLoading, loadNextBatch]);

  return (
    <div>
      <div
        ref={containerRef}
        style={{
          display: 'flex',
          flexDirection: 'row',
          width: '100%',
          gap: `${COLUMNS_GAP}px`,
        }}
      >
        {columns.map((columnItems, columnIndex) => (
          <div
            key={columnIndex}
            style={{
              display: 'flex',
              flexDirection: 'column',
              flexGrow: 1,
              gap: `${COLUMNS_GAP}px`,
              width: columnWidth ? `${columnWidth}px` : 'auto',
            }}
          >
            {columnItems.map(({ item, index }) => (
              <div key={index}>{renderItem(item, index)}</div>
            ))}
          </div>
        ))}
      </div>

      {/* Intersection observer for bottom of container */}
      <div ref={bottomObserverRef}>{isCurrentlyLoading && <div>Loading...</div>}</div>
    </div>
  );
}
