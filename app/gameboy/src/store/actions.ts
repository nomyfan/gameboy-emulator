import { storage } from "../storage/indexdb";

import type { IStore } from "./state";
import { store } from "./state";

export function selectCartridge(id?: string) {
  store.setState((state) => {
    state.selectedGameId = id;
  });
}

export function toggleSnapshotsDrawer(open?: boolean) {
  store.setState((st) => {
    st.ui.snapshotsDrawerOpen = open ?? !st.ui.snapshotsDrawerOpen;
  });
}

export function togglePlayModal(
  open?: true,
  snapshot?: IStore["snapshot"],
): void;
export function togglePlayModal(open?: false): void;
export function togglePlayModal(open?: boolean, snapshot?: IStore["snapshot"]) {
  store.setState((st) => {
    const playModalOpen = open ?? !st.ui.playModalOpen;
    st.ui.playModalOpen = playModalOpen;
    if (!playModalOpen) {
      const onClose = st.snapshot?.onClose;
      st.snapshot = undefined;
      onClose?.();
    } else if (snapshot) {
      st.snapshot = snapshot;
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

  store.setState((st) => {
    st.games = games?.filter((c) => c.id !== target.id);
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
