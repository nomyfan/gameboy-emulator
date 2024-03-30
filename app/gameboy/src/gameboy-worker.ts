import { GameBoy } from "./gameboy";

const handle = new GameBoy();

type U8 = number;

export type WorkerMessage =
  | {
      type: "install";
      payload: {
        buffer: Uint8ClampedArray;
        canvas: OffscreenCanvas;
        sampleRate: number;
        stream: WritableStream;
        scale?: number;
      };
    }
  | {
      type: "play";
    }
  | {
      type: "pause";
    }
  | {
      type: "change_key_state";
      payload: U8;
    };

self.onmessage = async (evt: MessageEvent<WorkerMessage>) => {
  const data = evt.data;
  if (data.type === "install") {
    handle.uninstall();
    const { buffer, canvas, sampleRate, stream, scale = 1 } = data.payload;
    const context = canvas.getContext("2d")!;
    context.setTransform(scale, 0, 0, scale, 0, 0);
    handle.install(buffer, canvas, sampleRate, stream);
    handle.play();
  } else if (data.type === "play") {
    handle.play();
  } else if (data.type === "pause") {
    handle.pause();
  } else if (data.type === "change_key_state") {
    handle.changeKeyState(data.payload);
  }
};

postMessage({ type: "ready" });
