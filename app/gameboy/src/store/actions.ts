import { storage } from "gameboy/storage/indexdb";

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
export function togglePlayModal(open?: false, invoke?: boolean): void;
export function togglePlayModal(
  open?: boolean,
  unknown?: boolean | IStore["snapshot"],
) {
  store.setState((st) => {
    let invoke: boolean | undefined;
    let snapshot: IStore["snapshot"] | undefined;
    if (open === undefined) {
      open = !st.ui.playModalOpen;
    } else if (open) {
      snapshot = unknown as IStore["snapshot"] | undefined;
    } else {
      invoke = unknown as boolean | undefined;
    }

    st.ui.playModalOpen = open;
    if (open) {
      st.snapshot = snapshot;
    } else {
      const onClose = st.snapshot?.onClose;
      st.snapshot = undefined;
      invoke && onClose?.();
    }
  });
}

export function toggleExitGameModal(open?: true, onClose?: () => void): void;
export function toggleExitGameModal(open?: false, invoke?: boolean): void;
export function toggleExitGameModal(
  open?: boolean,
  unknown?: (() => void) | boolean,
) {
  store.setState((st) => {
    let onClose: (() => void) | undefined;
    let invoke: boolean | undefined;
    if (open === undefined) {
      open = !st.ui.exitModalOpen;
    } else if (open) {
      onClose = unknown as (() => void) | undefined;
    } else {
      invoke = unknown as boolean | undefined;
    }

    st.ui.exitModalOpen = open;
    if (open) {
      st.ui.exitModalOnClose = onClose;
    } else {
      const onClose = st.ui.exitModalOnClose;
      st.ui.exitModalOnClose = undefined;
      invoke && onClose?.();
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
