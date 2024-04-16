import { mockGames } from "../components/GameList/assets/cover";
import * as storage from "../fs/storage";

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
  store.setState((st) => {
    st.games.cartridges = manifests.map((manifest) => {
      // TODO:
      const coverURL = mockGames[0];
      return {
        id: manifest.directory,
        path: manifest.directory,
        coverURL: coverURL,
        name: manifest.name,
      };
    });
  });
}

export async function deleteGame(id: string) {
  const cartridges = store.getState().games.cartridges;
  const target = cartridges?.find((c) => c.id === id);
  if (!target) {
    return;
  }

  await storage.deleteGame(target.path);

  store.setState((state) => {
    state.games.cartridges = cartridges?.filter((c) => c.id !== id);
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
