import { ModalCanceledError } from "gameboy/model/error";
import { storage } from "gameboy/storage/indexdb";

import type { IStore } from "./state";
import { store } from "./state";

export function selectCartridge(id?: string) {
  store.setState((state) => {
    state.selectedGameId = id;
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

export function openConfirmModal(options: {
  title: string;
  content: string;
  okText?: string;
  cancelText?: string;
  /**
   * @default true
   */
  ignoreCancel?: boolean;
}) {
  return new Promise<void>((resolve, reject) => {
    const onClose = (ok: boolean) => {
      store.setState((st) => {
        st.dialog.confirm = { open: false };
      });

      const ignoreCancel = options.ignoreCancel ?? true;
      ok ? resolve() : !ignoreCancel && reject(new ModalCanceledError());
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

export async function openSettingsModal() {
  return new Promise<void>((resolve) => {
    store.setState((st) => {
      st.dialog.settings.open = true;
      st.dialog.settings.callback = () => {
        store.setState((st) => {
          st.dialog.settings = { open: false };
        });

        resolve();
      };
    });
  });
}

export function closeSettingsModal() {
  store.getState().dialog.settings.callback?.();
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

export function writeSettings(settings: IStore["settings"]) {
  store.setState((st) => {
    st.settings = settings;
  });
  localStorage.setItem("gbos-settings", JSON.stringify(settings));
}
