import { isPlainObject } from "@callcc/toolkit-js/isPlainObject";
import { create } from "gameboy/store/utils";

export type ISettings = {
  volume: number;
  /**
   * Pause games automatically when the tab is not active.
   */
  autoPause: boolean;
  /**
   * Ignore the compatibility colors for DMG games.
   */
  coerceBwColors: boolean;
};

function createStore() {
  const patch = <T extends object>(target: T, source: Partial<T>): T => {
    if (!target || !source) {
      return target;
    }

    const targetKeys = Object.keys(target);
    for (const [key, value] of Object.entries(source)) {
      if (targetKeys.includes(key)) {
        // @ts-ignore
        if (isPlainObject(value) && isPlainObject(target[key])) {
          // @ts-ignore
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

  return create(() => {
    const settingsStr = localStorage.getItem("gbos-settings");
    return patch<ISettings>(
      { volume: 100, autoPause: true, coerceBwColors: false },
      settingsStr ? JSON.parse(settingsStr) : null,
    );
  });
}

const store = createStore();
export { store as settingsStore };

export function writeSettings(settings: ISettings) {
  store.setState(settings);
  localStorage.setItem("gbos-settings", JSON.stringify(settings));
}
