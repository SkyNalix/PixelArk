import { Fragment, useCallback, useEffect, useMemo, useRef, RefObject, CSSProperties } from 'react';
import { useGallery } from '@/app/gallery/galleryContext/useGallery.ts';
import { ChevronRight } from 'lucide-react';
import { listen } from '@tauri-apps/api/event';

function scrollIntoView(ref: RefObject<HTMLDivElement>) {
  if (ref.current) {
    ref.current.scrollIntoView({
      behavior: 'instant',
    });
  }
}

export function FolderBreadcrumb() {
  const { currentDirectory, setCurrentDirectory } = useGallery();

  const lastItemRef = useRef<HTMLDivElement>(null);

  const rootItem = useMemo(() => {
    const style: CSSProperties =
      currentDirectory.length == 0
        ? {}
        : {
            cursor: 'pointer',
          };
    return (
      <button
        onClick={() => {
          setCurrentDirectory([]);
        }}
        style={style}
      >
        Root
      </button>
    );
  }, [currentDirectory.length, setCurrentDirectory]);

  const changeDirectory = useCallback(
    (index: number) => {
      setCurrentDirectory((prev) => prev.slice(0, index + 1));
    },
    [setCurrentDirectory],
  );

  const items = useMemo(() => {
    return currentDirectory.map((directory, index) => {
      const isCurrentDirectory = index === currentDirectory.length - 1;
      let item;
      if (isCurrentDirectory) {
        item = <div ref={lastItemRef}>{directory}</div>;
      } else {
        item = (
          <button
            onClick={() => changeDirectory(index)}
            style={{
              cursor: 'pointer',
            }}
          >
            {directory}
          </button>
        );
      }
      return <Fragment key={index}>{item}</Fragment>;
    });
  }, [changeDirectory, currentDirectory]);

  useEffect(() => {
    scrollIntoView(lastItemRef);
  }, [items]);

  // scroll to the last breadcrumb element when window is resized
  useEffect(() => {
    const listener = listen('tauri://resize', async () => scrollIntoView(lastItemRef));
    return () => {
      listener.then((fn) => fn()); // clean up
    };
  }, []);

  return (
    <div
      style={{
        width: '100%',
        display: 'flex',
        flexDirection: 'row',
        gap: '6px',
        overflowX: 'scroll',
        textWrap: 'nowrap',
        scrollbarWidth: 'none',
      }}
    >
      <div
        style={{
          flexShrink: 0,
        }}
      >
        {rootItem}
      </div>
      {items.map((item, index) => (
        <div
          key={index}
          style={{
            display: 'flex',
            flexShrink: 0,
            alignItems: 'center',
            gap: '6px',
          }}
        >
          <ChevronRight
            absoluteStrokeWidth
            style={{
              width: 16,
              height: 16,
              flexShrink: 0,
            }}
          />
          {item}
        </div>
      ))}
    </div>
  );
}
