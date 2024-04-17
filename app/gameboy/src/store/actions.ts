import * as fs from "../fs";
import * as storage from "../fs/storage";

import type { IStore } from "./state";
import { store } from "./state";

export function selectCartridge(id?: string) {
  store.setState((state) => {
    state.ui.selectedCartridgeId = id;
  });
}

export function toggleSnapshotsDrawer(open?: boolean) {
  store.setState((state) => {
    state.ui.snapshotsDrawerOpen = open ?? !state.ui.snapshotsDrawerOpen;
  });
}

export async function loadGames(beforeSetState?: () => Promise<void>) {
  const manifests = await storage.loadAllGames();
  await beforeSetState?.();

  const cartridges: IStore["games"]["cartridges"] = [];
  for (const manifest of manifests) {
    const cover = await fs.file("cover.jpeg", await fs.dir(manifest.directory));
    const coverURL = await cover.getFile().then((file) => {
      return URL.createObjectURL(file);
    });
    cartridges.push({
      id: manifest.directory,
      path: manifest.directory,
      coverURL: coverURL,
      name: manifest.name,
    });
  }
  store.setState((st) => {
    st.games.cartridges = cartridges;
  });
}

export async function deleteGame(id: string) {
  const cartridges = store.getState().games.cartridges;
  const target = cartridges?.find((c) => c.id === id);
  if (!target) {
    return;
  }

  await storage.uninstallGame(target.path);

  store.setState((state) => {
    state.games.cartridges = cartridges?.filter((c) => c.id !== target.id);
    URL.revokeObjectURL(target.coverURL);
  });
}

export async function deleteSelectedGame() {
  const id = store.getState().ui.selectedCartridgeId;
  if (!id) {
    return;
  }

  await deleteGame(id);
  selectCartridge();
}
