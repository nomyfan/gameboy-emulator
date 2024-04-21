import { GameBoy as GameBoyHandle, JoypadKey } from "gb-wasm";
import { createStore } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";

function createGameBoyStore() {
  return createStore(
    subscribeWithSelector(
      immer(() => {
        return {
          status: "uninstalled" as
            | "playing"
            | "paused"
            | "installed"
            | "uninstalled",
        };
      }),
    ),
  );
}

type GameBoyStore = ReturnType<typeof createGameBoyStore>;

class GameBoyControl {
  private readonly store_: GameBoyStore;

  private instance_?: GameBoyHandle;
  private playCallbackId_?: number;

  private keyState = 0;

  constructor() {
    this.store_ = createGameBoyStore();
  }

  private get state() {
    type ReadonlyState = Readonly<ReturnType<(typeof this.store_)["getState"]>>;
    return this.store_.getState() as ReadonlyState;
  }

  get store() {
    return this.store_;
  }

  // TODO: improve typing to help the caller known this.instance_ is not undefined
  private ensureInstalled() {
    if (!this.instance_) {
      throw new Error("GameBoy is not installed");
    }
  }

  install(
    rom: Uint8ClampedArray,
    canvas: HTMLCanvasElement,
    scale?: number,
    sav?: Uint8Array,
  ) {
    this.instance_ = GameBoyHandle.create(rom, canvas, scale, sav);
    this.store_.setState({ status: "installed" });
  }

  uninstall() {
    if (this.instance_) {
      this.pause();
    }
    if (this.instance_) {
      this.instance_.free();
      this.instance_ = undefined;
    }
    this.store_.setState({ status: "uninstalled" });
  }

  play() {
    this.ensureInstalled();
    if (this.state.status === "playing") {
      return;
    }

    this.store_.setState({ status: "playing" });

    const playCallback = () => {
      if (this.state.status !== "playing" || !this.instance_) {
        return;
      }

      const start = performance.now();
      this.instance_!.continue();
      const duration = performance.now() - start;

      this.playCallbackId_ = setTimeout(() => {
        playCallback();
      }, 16.6 - duration) as unknown as number;
    };
    playCallback();
  }

  pause() {
    this.ensureInstalled();
    this.store_.setState({ status: "paused" });

    if (this.playCallbackId_) {
      clearTimeout(this.playCallbackId_);
      this.playCallbackId_ = undefined;
    }
  }

  changeKey(key: JoypadKey, pressed: boolean) {
    if (this.state.status === "playing") {
      let newState = this.keyState;
      if (pressed) {
        newState |= key;
      } else {
        newState &= ~key;
      }
      this.keyState = newState;
      this.instance_!.changeKeyState(newState);
    }
  }

  changeKeyState(state: number) {
    if (this.state.status === "playing") {
      this.keyState = state;
    }
  }

  takeSnapshot() {
    this.ensureInstalled();
    return this.instance_!.takeSnapshot();
  }

  restoreSnapshot(snapshot: Uint8Array) {
    this.ensureInstalled();
    this.instance_!.restoreSnapshot(snapshot);
  }

  createSav() {
    this.ensureInstalled();
    return this.instance_!.suspendCartridge();
  }
}

export { GameBoyControl, JoypadKey };
