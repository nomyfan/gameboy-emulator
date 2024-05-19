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
    settings: {
      open?: boolean;
      callback?: () => void;
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
  settings: {
    volume: number;
  };
}

export const store = create<IStore>(() => {
  const settings = localStorage.getItem("gbos-settings");
  return {
    dialog: {
      play: {},
      confirm: {},
      settings: {},
    },
    settings: settings ? JSON.parse(settings) : { volume: 100 },
  };
});

export function useAppStore<T>(selector: (state: Readonly<IStore>) => T) {
  return useStore(store, selector);
}
