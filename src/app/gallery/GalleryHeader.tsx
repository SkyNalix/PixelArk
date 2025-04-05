import { MAIN_HEADER_HEIGHT } from '@/app/constants.ts';
import { SidebarTrigger } from '@/components/ui/sidebar.tsx';
import { Button } from '@/components/ui/button.tsx';

export function GalleryHeader() {
  return (
    <div
      style={{
        height: `${MAIN_HEADER_HEIGHT}px`,
        width: '100%',
        display: 'flex',
        flexDirection: 'row',
        gap: 5,

        alignItems: 'center',
        backgroundColor: 'var(--base)',
        borderBottom: '2px solid var(--border)',
      }}
    >
      <Button variant={'secondary'} size={'icon'}>
        <SidebarTrigger />
      </Button>
    </div>
  );
}
