import { createStore } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";

export interface IStore {
  ui: {
    selectedCartridgeId?: string;
    snapshotsDrawerOpen?: boolean;
  };
  games: {
    cartridges?: {
      id: string;
      /**
       * Path in OPFS
       */
      path: string;
      coverURL: string;
      name: string;
    }[];
  };
}

function create() {
  return createStore(
    subscribeWithSelector(
      immer<IStore>(() => {
        return {
          ui: {},
          games: {},
        };
      }),
    ),
  );
}

export const store = create();
