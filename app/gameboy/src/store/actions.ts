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
  let invoke: boolean | undefined;
  let snapshot: IStore["snapshot"] | undefined;
  if (open === undefined) {
    open = !store.getState().ui.playModalOpen;
  } else if (open) {
    snapshot = unknown as IStore["snapshot"] | undefined;
  } else {
    invoke = unknown as boolean | undefined;
  }

  if (!open && invoke) {
    store.getState().snapshot?.onClose?.();
  }

  store.setState((st) => {
    st.ui.playModalOpen = open;
    st.snapshot = open ? snapshot : undefined;
  });
}

type IExitGameModalOnCloseCallback = IStore["ui"]["exitModalOnClose"];
type IExitGameModalOnCloseAction = Parameters<
  NonNullable<IExitGameModalOnCloseCallback>
>[0];
export function toggleExitGameModal(
  open?: true,
  onClose?: IExitGameModalOnCloseCallback,
): void;
export function toggleExitGameModal(
  open?: false,
  action?: IExitGameModalOnCloseAction,
): void;
export function toggleExitGameModal(
  open?: boolean,
  unknown?: IExitGameModalOnCloseCallback | IExitGameModalOnCloseAction,
) {
  let onClose: IExitGameModalOnCloseCallback;
  let action: IExitGameModalOnCloseAction | undefined;
  if (open === undefined) {
    open = !store.getState().ui.exitModalOpen;
  } else if (open) {
    onClose = unknown as IExitGameModalOnCloseCallback | undefined;
  } else {
    action = unknown as IExitGameModalOnCloseAction | undefined;
  }

  if (!open && action) {
    store.getState().ui.exitModalOnClose?.(action);
  }

  store.setState((st) => {
    st.ui.exitModalOpen = open;
    st.ui.exitModalOnClose = open ? onClose : undefined;
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
