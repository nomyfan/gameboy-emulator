import * as fs from "../fs";
import * as storage from "../fs/storage";

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

export function togglePlayModal(open?: boolean) {
  store.setState((state) => {
    state.ui.playModalOpen = open ?? !state.ui.playModalOpen;
  });
}

export async function loadGames(beforeSetState?: () => Promise<void>) {
  const manifests = await storage.loadAllGames();
  await beforeSetState?.();

  const games: NonNullable<IStore["games"]> = [];
  for (const manifest of manifests) {
    const cover = await fs.file("cover.jpeg", await fs.dir(manifest.directory));
    const coverURL = await cover.getFile().then((file) => {
      return URL.createObjectURL(file);
    });
    games.push({
      id: manifest.directory,
      coverURL: coverURL,
      ...manifest,
    });
  }
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

  await storage.uninstallGame(target.directory);

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
