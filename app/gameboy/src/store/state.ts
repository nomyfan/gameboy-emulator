import { createStore } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";

import { IGameManifest } from "../types";

export interface IStore {
  ui: {
    snapshotsDrawerOpen?: boolean;
    playModalOpen?: boolean;
  };
  games?: (IGameManifest & { id: string; coverURL: string })[];
  selectedGameId?: string;
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
