import type { ISnapshot } from "gameboy/model";
import { storage } from "gameboy/storage/indexdb";
import { create } from "gameboy/store/utils";

export type IGameStore = {
  games?: Array<{
    id: string;
    name: string;
    time: number;
    cover: Blob;
    lastPlayTime?: number;
  }>;
};

function createStore() {
  return create<IGameStore>(() => ({}));
}

const store = createStore();

export { store as gameStore };

export async function loadGames(beforeSetState?: () => Promise<void>) {
  const manifests = await storage.loadAllGames();

  const games: NonNullable<IGameStore["games"]> = [];
  for (const manifest of manifests) {
    games.push({
      id: manifest.id,
      name: manifest.name,
      time: manifest.createTime,
      cover: manifest.cover,
      lastPlayTime: manifest.lastPlayTime,
    });
  }

  await beforeSetState?.();
  store.setState({ games });
}

export async function exportSnapshot(id: number, snapshot?: ISnapshot) {
  if (!snapshot) {
    // biome-ignore lint/style/noParameterAssign: await expressions cannot be used in a parameter initializer.
    snapshot = await storage.snapshotStore.queryById(id);
    if (!snapshot) {
      throw new Error("Snapshot not found");
    }
  }

  const gameName = store
    .getState()
    .games?.find((g) => g.id === snapshot.gameId)?.name;

  const { pack, filename } = await storage.exportGame(snapshot.gameId, {
    snapshots: [snapshot.id],
  });

  return { pack, filename: gameName ? `${gameName}-${filename}` : filename };
}

export async function deleteGame(id: string) {
  const games = store.getState().games;
  const target = games?.find((c) => c.id === id);
  if (!target) {
    return;
  }

  await storage.uninstallGame(target.id);

  store.setState((st) => {
    st.games = games?.filter((c) => c.id !== target.id);
  });
}
