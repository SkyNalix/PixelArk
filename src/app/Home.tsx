import { GalleryPanel } from '@/app/gallery/GalleryPanel.tsx';
import { attachConsole } from '@tauri-apps/plugin-log';
import { useEffect } from 'react';

function Home() {
  useEffect(() => {
    const connection = attachConsole();
    return () => {
      connection.then((fn) => fn()); // clean up
    };
  });

  return (
    <div
      style={{
        height: '100%',
        width: '100%',
        padding: 4,
        borderRadius: 'var(--radius)',
      }}
    >
      <GalleryPanel />
    </div>
  );
}

export default Home;
