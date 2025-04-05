import { Folder } from 'lucide-react';
import { OverflowTooltip } from '@/components/OverflowTooltip.tsx';

interface GalleryComponentProps {
  folderName: string;
  onClick: (folderName: string) => void;
}

export const GalleryFolder = ({ folderName, onClick }: GalleryComponentProps) => {
  return (
    <div
      style={{
        height: '200px',
        width: '200px',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',

        borderRadius: '8px',
        backgroundColor: 'var(--overlay0)',
        cursor: 'pointer',
      }}
      onClick={() => onClick(folderName)}
    >
      <Folder
        style={{
          width: '80px',
          height: '80px',
          color: 'var(--flamingo)',
          flexGrow: 1,
        }}
      />

      <div
        style={{
          width: '70%',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          marginBottom: '8px',
        }}
      >
        <OverflowTooltip>
          <div
            style={{
              color: 'var(--text)',
              textShadow: '2px 2px black',
            }}
          >
            {folderName}
          </div>
        </OverflowTooltip>
      </div>
    </div>
  );
};
