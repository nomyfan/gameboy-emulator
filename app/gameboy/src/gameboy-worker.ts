import { GameBoy } from "./gameboy";

const handle = new GameBoy();

self.onmessage = async (evt: MessageEvent<{ type: string; payload: any }>) => {
  const data = evt.data;
  if (data.type === "install") {
    handle.uninstall();
    const { buffer, offscreen, sampleRate, writableStream } = data.payload;
    const context = offscreen.getContext("2d")!;
    context.setTransform(3, 0, 0, 3, 0, 0);
    handle.install(buffer, offscreen, sampleRate, writableStream);
    handle.play();
  } else if (data.type === "play") {
    handle.play();
  } else if (data.type === "pause") {
    handle.pause();
  } else if (data.type === "change_key") {
    // TODO
  }
};
