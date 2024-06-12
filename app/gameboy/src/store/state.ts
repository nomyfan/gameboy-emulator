import { isPlainObject } from "gameboy/utils";
import { useStore } from "zustand";

import { create } from "./utils";

export interface IStore {
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
  games?: Array<{
    id: string;
    name: string;
    time: number;
    cover: Blob;
    lastPlayTime?: number;
  }>;
  selectedGameId?: string;
  settings: {
    volume: number;
    /**
     * Pause games automatically when the tab is not active.
     */
    autoPause: boolean;
  };
}

export const store = create<IStore>(() => {
  const settingsStr = localStorage.getItem("gbos-settings");

  const patch = <T extends object>(target: T, source: Partial<T>): T => {
    if (!target || !source) {
      return target;
    }

    const targetKeys = Object.keys(target);
    for (const [key, value] of Object.entries(source)) {
      if (targetKeys.includes(key)) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-expect-error
        if (isPlainObject(value) && isPlainObject(target[key])) {
          // eslint-disable-next-line @typescript-eslint/ban-ts-comment
          // @ts-expect-error
          target[key] = patch(target[key], value);
        } else {
          // TODO: Not handle the case where they have different types
          // eslint-disable-next-line @typescript-eslint/ban-ts-comment
          // @ts-expect-error
          target[key] = value;
        }
      }
    }

    return target;
  };

  const settings = patch<IStore["settings"]>(
    { volume: 100, autoPause: false },
    settingsStr ? JSON.parse(settingsStr) : null,
  );

  return {
    dialog: {
      play: {},
      confirm: {},
      settings: {},
    },
    settings,
  };
});

export function useAppStore<T>(selector: (state: Readonly<IStore>) => T) {
  return useStore(store, selector);
}
