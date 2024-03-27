import { RefObject, useRef, useState } from "react";
import { GameBoy } from "./gameboy";
import { useStore } from "zustand";

const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

const gameboyHandle = new GameBoy();

function Controller(props: { canvasRef: RefObject<HTMLCanvasElement> }) {
  const status = useStore(gameboyHandle.store, (state) => state.status);

  if (status === "uninstalled") {
    return null;
  }

  const statusText = status === "playing" ? "Pause" : "Play";
  const handleClick = () => {
    if (status === "playing") {
      gameboyHandle.pause();
    } else {
      // TODO: type safety
      gameboyHandle.play(props.canvasRef.current?.getContext("2d")!);
    }
  };

  return (
    <>
      <button onClick={handleClick}>{statusText}</button>
    </>
  );
}

function Monitor() {
  const fps = useStore(gameboyHandle.store, (state) => state.fps);

  return <div>FPS: {fps}</div>;
}

function App() {
  const ref = useRef<HTMLCanvasElement>(null);
  const [scale, setScale] = useState(2);

  return (
    <div>
      <canvas
        ref={ref}
        width={RESOLUTION_X * scale}
        height={RESOLUTION_Y * scale}
      />
      <br />
      <Monitor />
      <Controller canvasRef={ref} />
      <input
        type="file"
        accept=".gb"
        onChange={async (evt) => {
          const file = evt.target.files?.[0];
          if (!file) return;

          const canvas = ref.current!;
          const context = canvas.getContext("2d")!;
          context.setTransform(scale, 0, 0, scale, 0, 0);

          const raw_buffer = await file.arrayBuffer();
          const buffer = new Uint8ClampedArray(raw_buffer);

          gameboyHandle.uninstall();
          gameboyHandle.install(buffer);
          gameboyHandle.play(context);
        }}
      />
    </div>
  );
}

export default App;
