import type { GameBoy as GameBoyHandle, JoypadKey } from "gb_wasm_bindings";
import { newGameBoy } from "gb_wasm_bindings";
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

  private keyQueue_: { key: JoypadKey; pressed: boolean }[] = [];

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

  install(rom: Uint8ClampedArray) {
    this.instance_ = newGameBoy(rom);
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

  play(canvasContext: CanvasRenderingContext2D) {
    this.ensureInstalled();
    if (this.state.status === "playing") {
      return;
    }

    this.store_.setState({ status: "playing" });

    const drawCallback = () => {
      if (this.state.status !== "playing" || !this.instance_) {
        if (this.drawCallbackId_) {
          window.cancelAnimationFrame(this.drawCallbackId_);
          this.drawCallbackId_ = undefined;
        }
        this.monitor_.fps.stop();
        return;
      }

      if (this.instance_.draw(canvasContext)) {
        this.monitor_.fps.update();
      }
      this.drawCallbackId_ = window.requestAnimationFrame(drawCallback);
    };
    drawCallback();

    const playCallback = () => {
      if (this.state.status !== "playing" || !this.instance_) {
        return;
      }

      const start = performance.now();
      this.keyQueue_
        .splice(0, this.keyQueue_.length)
        .forEach(({ key, pressed }) => {
          this.instance_!.changeKey(key, pressed);
        });
      this.instance_!.playWithClocks();

      const duration = performance.now() - start;
      this.playCallbackId_ = window.setTimeout(() => {
        playCallback();
      }, Math.max(0, 16 - duration));
    };
    playCallback();
  }

  pause() {
    this.ensureInstalled();
    this.store_.setState({ status: "paused" });
    this.monitor_.fps.stop();

    if (this.playCallbackId_) {
      window.clearTimeout(this.playCallbackId_);
      this.playCallbackId_ = undefined;
    }
    if (this.drawCallbackId_) {
      window.cancelAnimationFrame(this.drawCallbackId_);
      this.drawCallbackId_ = undefined;
    }
  }

  changeKey(key: JoypadKey, pressed: boolean) {
    if (this.state.status === "playing") {
      this.keyQueue_.push({ key, pressed });
    }
  }
}

export { GameBoy };
