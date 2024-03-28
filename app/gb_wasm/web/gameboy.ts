import { GameBoy as GameBoyHandle, JoypadKey } from "gb_wasm_bindings";
import { createStore } from "zustand";
import { immer } from "zustand/middleware/immer";
import { subscribeWithSelector } from "zustand/middleware";

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
          fps: 0,
        };
      })
    )
  );
}

type GameBoyStore = ReturnType<typeof createGameBoyStore>;

class Fps {
  frameCount = 0;
  lastTime = performance.now();
  private store_: GameBoyStore;

  constructor(store: GameBoyStore) {
    this.store_ = store;
  }

  stop() {
    this.frameCount = 0;
  }

  update() {
    if (this.frameCount === 0) {
      this.frameCount = 1;
      this.lastTime = performance.now();
    }

    this.frameCount++;
    const now = performance.now();
    const duration = now - this.lastTime;
    if (duration >= 1000) {
      const fps = (this.frameCount - 1) / (duration / 1000);
      this.frameCount = 1;
      this.lastTime = now;

      postMessage({ type: "fps", payload: fps });

      this.store_.setState({ fps });
    }
  }
}

class Monitor {
  fps: Fps;
  constructor(store: GameBoyStore) {
    this.fps = new Fps(store);
  }
}

class GameBoy {
  private store_: GameBoyStore;
  private monitor_;

  private instance_?: GameBoyHandle;
  private playCallbackId_?: number;
  private drawCallbackId_?: number;

  private newKeyState_?: number;

  constructor() {
    this.store_ = createGameBoyStore();
    this.monitor_ = new Monitor(this.store_);
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
    canvas: OffscreenCanvas,
    sampleRate?: number,
    audioPort?: MessagePort
  ) {
    this.instance_ = GameBoyHandle.create(rom, canvas, sampleRate, audioPort);
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

    // TODO: can we just do this in the Rust side?
    // const drawCallback = () => {
    //   if (this.state.status !== "playing" || !this.instance_) {
    //     if (this.drawCallbackId_) {
    //       cancelAnimationFrame(this.drawCallbackId_);
    //       this.drawCallbackId_ = undefined;
    //     }
    //     this.monitor_.fps.stop();
    //     return;
    //   }

    //   if (this.instance_.draw(canvasContext)) {
    //     this.monitor_.fps.update();
    //   }
    //   this.drawCallbackId_ = requestAnimationFrame(drawCallback);
    // };
    // drawCallback();

    const playCallback = () => {
      if (this.state.status !== "playing" || !this.instance_) {
        return;
      }

      const start = performance.now();

      if (this.newKeyState_ !== undefined) {
        this.instance_!.changeKeyState(this.newKeyState_);
        this.newKeyState_ = undefined;
      }

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
    this.monitor_.fps.stop();

    if (this.playCallbackId_) {
      clearTimeout(this.playCallbackId_);
      this.playCallbackId_ = undefined;
    }
    if (this.drawCallbackId_) {
      cancelAnimationFrame(this.drawCallbackId_);
      this.drawCallbackId_ = undefined;
    }
  }

  changeKey(key: JoypadKey, pressed: boolean) {
    if (this.state.status === "playing") {
      let newState = this.newKeyState_ ?? 0;
      if (pressed) {
        newState |= key;
      } else {
        newState &= ~key;
      }
      this.newKeyState_ = newState;
    }
  }

  changeKeyState(state: number) {
    if (this.state.status === "playing") {
      this.newKeyState_ = state;
    }
  }
}

export { GameBoy };
