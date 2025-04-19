import { MAIN_HEADER_HEIGHT } from '@/app/constants.ts';
import { FolderBreadcrumb } from '@/app/gallery/FolderBreadcrumb.tsx';

export function GalleryHeader() {
  return (
    <div
      style={{
        height: `${MAIN_HEADER_HEIGHT}px`,
        width: '100%',
        display: 'flex',
        flexDirection: 'row',
        gap: 8,

        alignItems: 'center',
        backgroundColor: 'var(--base)',
        borderBottom: '2px solid var(--border)',
      }}
    >
      <FolderBreadcrumb />
    </div>
  );
}
