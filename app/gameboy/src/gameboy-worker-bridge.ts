// This is a module responds to coordinate the audio worklet and the main GameBoy emulator worker.

import type { WorkerMessage } from "./gameboy-worker";

function invoke(this: Worker, message: WorkerMessage) {
  this.postMessage(message);
}

class GameBoyBridge {
  private audioContext_: AudioContext;
  private worker_?: Worker;
  private workletNode_?: AudioWorkletNode;
  private invoke?: (message: WorkerMessage) => void;

  private constructor(audioContext: AudioContext) {
    this.audioContext_ = audioContext;
  }

  static async create(): Promise<GameBoyBridge> {
    const context = new AudioContext();
    await context.audioWorklet.addModule(
      new URL("./audio-worklet.js", import.meta.url),
    );

    return new GameBoyBridge(context);
  }

  async install(file: File, canvas: OffscreenCanvas, scale: number = 1) {
    const raw_buffer = await file.arrayBuffer();
    const buffer = new Uint8ClampedArray(raw_buffer);

    const sampleRate = this.audioContext_.sampleRate;
    // TODO: What if the sample rate changes due to device switching?
    const workletNode = new AudioWorkletNode(
      this.audioContext_,
      "GameBoyAudioProcessor",
      {
        numberOfOutputs: 1,
        outputChannelCount: [2],
        processorOptions: {
          sampleRate,
        },
      },
    );
    workletNode.connect(this.audioContext_.destination);
    this.workletNode_ = workletNode;

    const stream = await new Promise<WritableStream>((resolve) => {
      const handler = (evt: MessageEvent) => {
        if (evt.data.type === "stream") {
          workletNode.port.onmessage = null;
          resolve(evt.data.payload as WritableStream);
        }
      };
      workletNode.port.onmessage = handler;
    });

    const worker = new Worker(new URL("./gameboy-worker", import.meta.url), {
      type: "module",
    });

    await new Promise<void>((resolve) => {
      const onmessage = (evt: MessageEvent) => {
        if (evt.data.type === "ready") {
          worker.onmessage = null;
          resolve();
        }
      };
      worker.onmessage = onmessage;
    });

    worker.postMessage(
      {
        type: "install",
        payload: { buffer, canvas, sampleRate, stream, scale },
      },
      [canvas, stream],
    );

    this.worker_ = worker;
    this.invoke = invoke.bind(worker);
  }

  play() {
    this.invoke?.({ type: "play" });
  }

  pause() {
    this.invoke?.({ type: "pause" });
  }

  changeKeyState(state: number) {
    this.invoke?.({ type: "change_key_state", payload: state & 0xff });
  }

  async terminate() {
    // TODO: terminate
    // this.worker_.terminate();
    // this.workletNode_?.disconnect();
    // await this.audioContext_.close();
  }
}

export { GameBoyBridge };
