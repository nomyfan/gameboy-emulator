import { storage } from "../storage/indexdb";

import type { IStore } from "./state";
import { store } from "./state";

export function selectCartridge(id?: string) {
  store.setState((state) => {
    state.selectedGameId = id;
  });
}

export function toggleSnapshotsDrawer(open?: boolean) {
  store.setState((state) => {
    state.ui.snapshotsDrawerOpen = open ?? !state.ui.snapshotsDrawerOpen;
  });
}

export function togglePlayModal(
  open?: boolean,
  snapshot?: IStore["snapshot"] | null,
) {
  store.setState((state) => {
    state.ui.playModalOpen = open ?? !state.ui.playModalOpen;
    if (snapshot === null) {
      state.snapshot = undefined;
    } else if (snapshot) {
      state.snapshot = snapshot;
    }
  });
}

export async function loadGames(beforeSetState?: () => Promise<void>) {
  const manifests = await storage.loadAllGames();

  const games: NonNullable<IStore["games"]> = [];
  for (const manifest of manifests) {
    const coverURL = URL.createObjectURL(manifest.cover);
    games.push({
      id: manifest.id,
      name: manifest.name,
      coverURL,
      time: manifest.createTime,
      lastPlayTime: manifest.lastPlayTime,
    });
  }

  await beforeSetState?.();
  store.setState((st) => {
    st.games = games;
  });
}

export async function deleteGame(id: string) {
  const games = store.getState().games;
  const target = games?.find((c) => c.id === id);
  if (!target) {
    return;
  }

  await storage.uninstallGame(target.id);

  store.setState((state) => {
    state.games = games?.filter((c) => c.id !== target.id);
    URL.revokeObjectURL(target.coverURL);
  });
}

export async function deleteSelectedGame() {
  const id = store.getState().selectedGameId;
  if (!id) {
    return;
  }

  await deleteGame(id);
  selectCartridge();
}
