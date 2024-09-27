import { ModalCanceledError } from "gameboy/model/error";
import { useStore } from "zustand";
import { deleteGame } from "./game";
import { create } from "./utils";

export type IStore = {
  dialog: {
    play: {
      open?: boolean;
      callback?: (action: "snapshot" | "no_snapshot") => void;
      snapshot?: {
        gameId: string;
        data: Uint8Array;
      };
    };
    // General confirm modal
    confirm: {
      open?: boolean;
      title?: string;
      content?: string;
      okText?: string;
      cancelText?: string;
      callback?: (ok: boolean) => void;
    };
    settings: {
      open?: boolean;
      callback?: () => void;
    };
  };
  selectedGameId?: string;
};

const store = create<IStore>(() => {
  return {
    dialog: {
      play: {},
      confirm: {},
      settings: {},
    },
  };
});

export { store as appStore };

export function useAppStore<T>(selector: (state: Readonly<IStore>) => T) {
  return useStore(store, selector);
}

export function selectCartridge(id?: string) {
  store.setState((state) => {
    state.selectedGameId = id;
  });
}

type IPlayModalCallback = NonNullable<IStore["dialog"]["play"]["callback"]>;
type IPlayModalCallbackAction = Parameters<IPlayModalCallback>[0];
export function openPlayModal(snapshot?: IStore["dialog"]["play"]["snapshot"]) {
  return new Promise<IPlayModalCallbackAction>((resolve) => {
    store.setState((st) => {
      st.dialog.play = {
        open: true,
        callback: (action) => {
          store.setState((st) => {
            st.dialog.play = { open: false };
          });

          resolve(action);
        },
        snapshot,
      };
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
      st.dialog.settings = {
        open: true,
        callback: () => {
          store.setState((st) => {
            st.dialog.settings = { open: false };
          });

          resolve();
        },
      };
    });
  });
}

export function closeSettingsModal() {
  store.getState().dialog.settings.callback?.();
}

export async function deleteSelectedGame() {
  const id = store.getState().selectedGameId;
  if (!id) {
    return;
  }

  await deleteGame(id);
  selectCartridge();
}
