import { ReactElement } from 'react';

type GridViewProps<T> = {
  items: T[];
  renderItem: (item: T, index: number) => ReactElement;
};

export function GridView<T>({ items, renderItem }: GridViewProps<T>): ReactElement {
  if (items.length === 0) {
    return <></>;
  }

  return (
    <div
      style={{
        width: '100%',
        display: 'flex',
        flexDirection: 'row',
        flexWrap: 'wrap',
        gap: '8px',
        paddingBottom: '20px',
      }}
    >
      {items.map((item, index) => (
        <div key={index}>{renderItem(item, index)}</div>
      ))}
    </div>
  );
}
