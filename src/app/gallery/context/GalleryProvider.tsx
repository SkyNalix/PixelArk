import { Dispatch, ReactNode, SetStateAction, useCallback, useEffect, useReducer, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ImageElementData } from '@/app/gallery/types.ts';
import { GalleryContext } from './GalleryContext';

const BATCH_SIZE = 30;

export interface GalleryContextType {
  // States
  medias: ImageElementData[];
  isCurrentlyLoading: boolean;
  noMoreBatches: boolean;
  folderNames: string[];
  currentDirectory: string[];

  // Methods
  loadNextBatch: () => void;
  setCurrentDirectory: Dispatch<SetStateAction<string[]>>;
}

type DirectoryState = {
  folderNames: string[] | null;
  medias: ImageElementData[];
  isLoading: boolean;
  noMoreBatches: boolean;
};

const defaultDirectoryState: DirectoryState = {
  folderNames: [],
  medias: [],
  isLoading: false,
  noMoreBatches: false,
};

type GalleryState = Record<string, DirectoryState>;

type Action =
  | { type: 'SET_LOADING'; directory: string; value: boolean }
  | { type: 'ADD_MEDIAS'; directory: string; medias: ImageElementData[]; noMoreBatches: boolean }
  | { type: 'SET_FOLDER_NAMES'; directory: string; folderNames: string[] }
  | { type: 'RESET_DIRECTORY'; directory: string };

function reducer(state: GalleryState, action: Action): GalleryState {
  const dir = action.directory;
  const prev = state[dir] || defaultDirectoryState;

  switch (action.type) {
    case 'SET_LOADING':
      return { ...state, [dir]: { ...prev, isLoading: action.value } };
    case 'SET_FOLDER_NAMES':
      return { ...state, [dir]: { ...prev, folderNames: action.folderNames } };
    case 'ADD_MEDIAS':
      return {
        ...state,
        [dir]: { ...prev, medias: [...prev.medias, ...action.medias], noMoreBatches: action.noMoreBatches },
      };
    case 'RESET_DIRECTORY':
      return { ...state, [dir]: defaultDirectoryState };
    default:
      return state;
  }
}

export function GalleryProvider({ children }: { children: ReactNode }) {
  const [currentDirectory, setCurrentDirectory] = useState<string[]>([]);
  const [state, dispatch] = useReducer(reducer, {});
  const lastLoadTimeRef = useRef<number>(0);

  const dirKey = currentDirectory.join('/');

  const loadNextBatch = useCallback(async () => {
    const now = Date.now();
    if (now - lastLoadTimeRef.current < 2000) return;

    const current = state[dirKey] || defaultDirectoryState;
    if (current.isLoading || current.noMoreBatches) return;

    lastLoadTimeRef.current = now;
    dispatch({ type: 'SET_LOADING', directory: dirKey, value: true });

    try {
      const start = current.medias.length;
      const stop = start + BATCH_SIZE;

      const { medias, no_more_batches: noMoreBatches } = await invoke<{
        medias: ImageElementData[];
        no_more_batches: boolean;
      }>('load_images_from_directory', {
        directory: dirKey,
        start,
        stop,
      });

      dispatch({ type: 'ADD_MEDIAS', directory: dirKey, medias, noMoreBatches });
    } catch (err) {
      console.error(err);
    } finally {
      dispatch({ type: 'SET_LOADING', directory: dirKey, value: false });
    }
  }, [dirKey, state]);

  useEffect(() => {
    lastLoadTimeRef.current = 0;

    if (state[dirKey]?.folderNames == null) {
      invoke<string[]>('get_folder_names', { directory: dirKey })
        .then((names) => {
          dispatch({ type: 'SET_FOLDER_NAMES', directory: dirKey, folderNames: names });
        })
        .catch((err) => {
          console.error(err);
        });
    }

    // we only want to run this once when the current directory is modified
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dirKey]);

  const value: GalleryContextType = {
    medias: state[dirKey]?.medias || [],
    isCurrentlyLoading: state[dirKey]?.isLoading || false,
    noMoreBatches: state[dirKey]?.noMoreBatches || false,
    folderNames: state[dirKey]?.folderNames || [],
    currentDirectory,
    loadNextBatch,
    setCurrentDirectory,
  };

  return <GalleryContext.Provider value={value}>{children}</GalleryContext.Provider>;
}
