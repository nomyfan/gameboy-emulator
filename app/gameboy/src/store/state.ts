import { createStore } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";

import mockGame1 from "../components/game-list/assets/game1.jpeg";
import mockGame2 from "../components/game-list/assets/game2.png";
import mockGame3 from "../components/game-list/assets/game3.jpeg";
import mockGame4 from "../components/game-list/assets/game4.jpeg";
import mockGame5 from "../components/game-list/assets/game5.jpeg";

const mockGames = [
  {
    id: "id1",
    path: "path1",
    coverURL: mockGame1,
    name: "Game 1",
  },
  {
    id: "id2",
    path: "path2",
    coverURL: mockGame2,
    name: "Game 2",
  },
  {
    id: "id3",
    path: "path3",
    coverURL: mockGame3,
    name: "Game 3",
  },
  {
    id: "id4",
    path: "path4",
    coverURL: mockGame4,
    name: "Game 4",
  },
  {
    id: "id5",
    path: "path5",
    coverURL: mockGame5,
    name: "Game 5",
  },
];

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
