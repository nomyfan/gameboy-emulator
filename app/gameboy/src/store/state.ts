import { createStore, useStore } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";

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
    snapshot: {
      open?: boolean;
    };
    exitGameConfirm: {
      open?: boolean;
      callback?: (action: "snapshot" | "no_snapshot" | "cancel") => void;
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

function create() {
  return createStore(
    subscribeWithSelector(
      immer<IStore>(() => {
        return {
          dialog: {
            play: {},
            snapshot: {},
            exitGameConfirm: {},
            confirm: {},
          },
        };
      }),
    ),
  );
}

export const store = create();

export function useAppStore<T>(selector: (state: Readonly<IStore>) => T) {
  return useStore(store, selector);
}
