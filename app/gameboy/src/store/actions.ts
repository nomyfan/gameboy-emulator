import { ModalCanceledError } from "gameboy/model/error";
import { storage } from "gameboy/storage/indexdb";

import type { IStore } from "./state";
import { store } from "./state";

export function selectCartridge(id?: string) {
  store.setState((state) => {
    state.selectedGameId = id;
  });
}

export function toggleSnapshotModal(open?: boolean) {
  store.setState((st) => {
    st.ui.snapshotModalOpen = open ?? !st.ui.snapshotModalOpen;
  });
}

type IPlayModalCallback = NonNullable<IStore["ui"]["playModalCallback"]>;
type IPlayModalCallbackAction = Parameters<IPlayModalCallback>[0];
export function openPlayModal(snapshot?: IStore["snapshot"]) {
  return new Promise<IPlayModalCallbackAction>((resolve) => {
    const onClose: IPlayModalCallback = (action) => {
      store.setState((st) => {
        st.ui.playModalOpen = false;
        st.ui.playModalCallback = undefined;
        st.snapshot = undefined;
      });

      resolve(action);
    };

    store.setState((st) => {
      st.ui.playModalOpen = true;
      st.ui.playModalCallback = onClose;
      st.snapshot = snapshot;
    });
  });
}

export function closePlayModal(action: IPlayModalCallbackAction) {
  store.getState().ui.playModalCallback?.(action);
}

type IExitGameModalCallback = IStore["ui"]["confirmExitModalCallback"];
type IExitGameModalCallbackAction = Parameters<
  NonNullable<IExitGameModalCallback>
>[0];
export function openConfirmExitModal() {
  return new Promise<Exclude<IExitGameModalCallbackAction, "cancel">>(
    (resolve, reject) => {
      const onClose: NonNullable<IExitGameModalCallback> = (action) => {
        store.setState((st) => {
          st.ui.confirmExitModalOpen = false;
          st.ui.confirmExitModalCallback = undefined;
        });

        if (action === "cancel") {
          reject(new ModalCanceledError());
        } else {
          resolve(action);
        }
      };
      store.setState((st) => {
        st.ui.confirmExitModalOpen = true;
        st.ui.confirmExitModalCallback = onClose;
      });
    },
  );
}

export function closeConfirmExitModal(action: IExitGameModalCallbackAction) {
  store.getState().ui.confirmExitModalCallback?.(action);
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
