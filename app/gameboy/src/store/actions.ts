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
    st.dialog.snapshot.open = open ?? !st.dialog.snapshot.open;
  });
}

type IPlayModalCallback = NonNullable<IStore["dialog"]["play"]["callback"]>;
type IPlayModalCallbackAction = Parameters<IPlayModalCallback>[0];
export function openPlayModal(snapshot?: IStore["dialog"]["play"]["snapshot"]) {
  return new Promise<IPlayModalCallbackAction>((resolve) => {
    const onClose: IPlayModalCallback = (action) => {
      store.setState((st) => {
        st.dialog.play.open = false;
        st.dialog.play.callback = undefined;
        st.dialog.play.snapshot = undefined;
      });

      resolve(action);
    };

    store.setState((st) => {
      st.dialog.play.open = true;
      st.dialog.play.callback = onClose;
      st.dialog.play.snapshot = snapshot;
    });
  });
}

export function closePlayModal(action: IPlayModalCallbackAction) {
  store.getState().dialog.play.callback?.(action);
}

type IExitGameModalCallback = IStore["dialog"]["exitGameConfirm"]["callback"];
type IExitGameModalCallbackAction = Parameters<
  NonNullable<IExitGameModalCallback>
>[0];
export function openExitConfirmModal() {
  return new Promise<Exclude<IExitGameModalCallbackAction, "cancel">>(
    (resolve, reject) => {
      const onClose: NonNullable<IExitGameModalCallback> = (action) => {
        store.setState((st) => {
          st.dialog.exitGameConfirm.open = false;
          st.dialog.exitGameConfirm.callback = undefined;
        });

        if (action === "cancel") {
          reject(new ModalCanceledError());
        } else {
          resolve(action);
        }
      };
      store.setState((st) => {
        st.dialog.exitGameConfirm.open = true;
        st.dialog.exitGameConfirm.callback = onClose;
      });
    },
  );
}

export function closeExitConfirmModal(action: IExitGameModalCallbackAction) {
  store.getState().dialog.exitGameConfirm.callback?.(action);
}

export function openConfirmModal(options: {
  title: string;
  content: string;
  okText?: string;
  cancelText?: string;
}) {
  return new Promise<void>((resolve, reject) => {
    const onClose = (ok: boolean) => {
      store.setState((st) => {
        st.dialog.confirm = { open: false };
      });

      ok ? resolve() : reject(new ModalCanceledError());
    };

    store.setState((st) => {
      st.dialog.confirm = {
        open: true,
        callback: onClose,
        title: options.title,
        content: options.content,
        okText: options.okText,
        cancelText: options.cancelText,
      };
    });
  });
}

export function closeConfirmModal(ok: boolean) {
  store.getState().dialog.confirm.callback?.(ok);
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

export async function exportSelectedGame() {
  const id = store.getState().selectedGameId;
  if (!id) {
    throw new Error();
  }

  return await storage.exportGame(id);
}
