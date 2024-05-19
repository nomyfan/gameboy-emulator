import { useStore } from "zustand";

import { create } from "./utils";

export interface IStore {
  dialog: {
    play: {
      open?: boolean;
      callback?: (action: "snapshot" | "no_snapshot") => void;
      snapshot?: {
        gameId: string;
        data: Uint8Array;
      };
    };
    // General confirm modal
    confirm: {
      open?: boolean;
      title?: string;
      content?: string;
      okText?: string;
      cancelText?: string;
      callback?: (ok: boolean) => void;
    };
  };
  games?: Array<{
    id: string;
    name: string;
    time: number;
    coverURL: string;
    lastPlayTime?: number;
  }>;
  selectedGameId?: string;
}

export const store = create<IStore>(() => ({
  dialog: {
    play: {},
    confirm: {},
  },
}));

export function useAppStore<T>(selector: (state: Readonly<IStore>) => T) {
  return useStore(store, selector);
}
