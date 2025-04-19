import { GalleryPanel } from '@/app/gallery/GalleryPanel.tsx';

function Home() {
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
