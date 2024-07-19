import { store } from "gameboy/store";
import { create } from "gameboy/store/utils";
import { GameBoy as GameBoyHandle, JoypadButton } from "gb_wasm";

function noop() {}

function createGameBoyStore() {
  return create<{
    status: "playing" | "paused" | "installed" | "uninstalled";
    muted: boolean;
    volume: number;
  }>(() => ({ status: "uninstalled", muted: false, volume: 1 }));
}

type GameBoyStore = ReturnType<typeof createGameBoyStore>;

class GameBoyControl {
  private readonly store_: GameBoyStore;

  private instance_?: GameBoyHandle;
  private playCallbackId_?: number;

  private buttonsState = 0;
  private audioContext_?: AudioContext;
  private audioWorkletModuleAdded_ = false;
  private disposeAudio_: () => void = noop;
  private changeAudioVolume_: (volume: number) => void = noop;
  private nextTickTime_ = 0;

  constructor() {
    this.store_ = createGameBoyStore();
    this.store_.setState({ volume: store.getState().settings.volume });
    // Subscribe to global store volume changes
    store.subscribe((state) => {
      if (state.settings.volume !== this.state.volume) {
        this.store_.setState({ volume: state.settings.volume });
        this.changeAudioVolume_(state.settings.volume);
      }
    });
  }

  get state() {
    type ReadonlyState = Readonly<ReturnType<(typeof this.store_)["getState"]>>;
    return this.store_.getState() as ReadonlyState;
  }

  get store() {
    return this.store_;
  }

  private async setupAudio() {
    this.audioContext_ = this.audioContext_ ?? new AudioContext();
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

    const gainNode = this.audioContext_.createGain();
    gainNode.gain.value = this.state.volume / 100;
    workletNode.connect(gainNode);
    gainNode.connect(this.audioContext_.destination);
    this.disposeAudio_ = () => {
      workletNode.disconnect();
      gainNode.disconnect();

      this.disposeAudio_ = noop;
      this.changeAudioVolume_ = noop;
    };
    this.changeAudioVolume_ = (volume: number) => {
      gainNode.gain.value = volume / 100;
    };

    return { stream, sampleRate };
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
    sav?: Uint8Array,
    dbgCanvas?: HTMLCanvasElement,
  ) {
    const { stream, sampleRate } = await this.setupAudio();
    const instance = GameBoyHandle.create(
      rom,
      canvas,
      sav,
      sampleRate,
      stream,
      dbgCanvas,
    );
    this.instance_ = instance;
    instance.mute(this.state.muted);
    this.store_.setState({ status: "installed" });
  }

  uninstall() {
    if (this.instance_) {
      this.pause();
    }

    this.disposeAudio_();

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
      const nextTickTime = start + 17 - delayed;
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

  changeButton(button: JoypadButton, pressed: boolean) {
    if (this.state.status === "playing") {
      let newState = this.buttonsState;
      if (pressed) {
        newState |= button;
      } else {
        newState &= ~button;
      }
      this.buttonsState = newState;
      this.instance_!.mutateButtons(newState);
    }
  }

  changeButtons(state: number) {
    if (this.state.status === "playing") {
      this.buttonsState = state;
      this.instance_!.mutateButtons(state);
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

  mute(muted?: boolean) {
    muted ??= !this.state.muted;
    this.store_.setState({ muted });
    this.instance_?.mute(muted);
  }
}

export { GameBoyControl, JoypadButton };
