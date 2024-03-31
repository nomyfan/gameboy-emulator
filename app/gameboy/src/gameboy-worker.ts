import init from "gb-wasm";

import { GameBoyControl } from "./gameboy";

const control = new GameBoyControl();

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
    control.uninstall();
    const { buffer, canvas, sampleRate, stream } = data.payload;
    control.install(
      buffer,
      canvas,
      data.payload.scale || 1,
      sampleRate && stream ? { sampleRate, stream } : undefined,
    );
    control.play();
  } else if (data.type === "play") {
    control.play();
  } else if (data.type === "pause") {
    control.pause();
  } else if (data.type === "change_key_state") {
    control.changeKeyState(data.payload);
  }
};

init().then(() => {
  postMessage({ type: "ready" });
});
