import { useRef, useState } from "react";
import { GameBoy } from "./gameboy";

const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

const gameboyHandle = new GameBoy();

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
      <div id="fps">FPS: 0</div>
      <button
        onClick={() => {
          gameboyHandle.pause();
        }}
      >
        Pause
      </button>
      <button
        onClick={() => {
          gameboyHandle.play(ref.current!.getContext("2d")!);
        }}
      >
        Play
      </button>
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
