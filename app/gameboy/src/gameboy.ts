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
  private readonly audioContext_: AudioContext;
  private audioWorkletModuleAdded_ = false;
  private audioWorkletNode_?: AudioWorkletNode;
  private nextTickTime_ = 0;

  constructor() {
    this.store_ = createGameBoyStore();
    this.audioContext_ = new AudioContext();
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

  async install(
    rom: Uint8ClampedArray,
    canvas: HTMLCanvasElement,
    scale?: number,
    sav?: Uint8Array,
  ) {
    if (!this.audioWorkletModuleAdded_) {
      await this.audioContext_.audioWorklet.addModule(
        new URL("./audio-worklet/gameboy-audio-processor.js", import.meta.url),
      );
      this.audioWorkletModuleAdded_ = true;
    }
    const sampleRate = this.audioContext_.sampleRate;
    const workletNode = new AudioWorkletNode(
      this.audioContext_,
      "gameboy-audio-processor",
      {
        numberOfOutputs: 1,
        outputChannelCount: [2], // Stereo
      },
    );
    this.audioWorkletNode_ = workletNode;
    workletNode.addEventListener("processorerror", (evt) => {
      console.error("processorerror", evt);
    });

    // Wait until audio worklet processor prepared
    const stream = await new Promise<WritableStream>((resolve) => {
      const handler = (evt: MessageEvent) => {
        if (evt.data.type === "stream-prepared") {
          workletNode.port.removeEventListener("message", handler);
          resolve(evt.data.payload as WritableStream);
        }
      };
      workletNode.port.addEventListener("message", handler);
      workletNode.port.start();
    });
    workletNode.connect(this.audioContext_.destination);

    this.instance_ = GameBoyHandle.create(
      rom,
      canvas,
      scale,
      sav,
      sampleRate,
      stream,
    );
    this.store_.setState({ status: "installed" });
  }

  uninstall() {
    if (this.instance_) {
      this.pause();
    }

    if (this.audioWorkletNode_) {
      this.audioWorkletNode_.disconnect();
      this.audioWorkletNode_ = undefined;
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
      const delayed = this.nextTickTime_ === 0 ? 0 : start - this.nextTickTime_;
      this.instance_!.continue();
      const nextTickTime = start + 16.666666 - delayed;
      this.nextTickTime_ = nextTickTime;

      this.playCallbackId_ = setTimeout(() => {
        playCallback();
      }, nextTickTime - performance.now()) as unknown as number;
    };
    playCallback();
  }

  pause() {
    this.ensureInstalled();
    this.store_.setState({ status: "paused" });

    if (this.playCallbackId_) {
      clearTimeout(this.playCallbackId_);
      this.playCallbackId_ = undefined;
      this.nextTickTime_ = 0;
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
      this.instance_!.changeKeyState(state);
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
