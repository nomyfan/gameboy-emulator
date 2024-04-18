import { createStore } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";

export interface IStore {
  ui: {
    snapshotsDrawerOpen?: boolean;
    playModalOpen?: boolean;
  };
  games?: Array<{
    id: string;
    name: string;
    time: number;
    coverURL: string;
    lastPlayTime?: number;
  }>;
  selectedGameId?: string;
  snapshot?: {
    gameId: string;
    data: Uint8Array;
  };
}

function create() {
  return createStore(
    subscribeWithSelector(
      immer<IStore>(() => {
        return {
          ui: {},
        };
      }),
    ),
  );
}

export const store = create();
