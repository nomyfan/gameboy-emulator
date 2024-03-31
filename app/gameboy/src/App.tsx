import { useRef, useState } from "react";

import { GameBoyBridge } from "./gameboy-worker-bridge";
import { useGamepadController } from "./hooks/useGamepadController";
import { useKeyboardController } from "./hooks/useKeyboardController";

const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

// const gameboyHandle = new GameBoy();

// function Controller(props: { canvasRef: RefObject<HTMLCanvasElement> }) {
//   const status = useStore(gameboyHandle.store, (state) => state.status);

//   if (status === "uninstalled") {
//     return null;
//   }

//   const statusText = status === "playing" ? "Pause" : "Play";
//   const handleClick = () => {
//     if (status === "playing") {
//       gameboyHandle.pause();
//     } else {
//       // TODO: type safety
//       gameboyHandle.play(props.canvasRef.current?.getContext("2d")!);
//     }
//   };

//   return (
//     <>
//       <button onClick={handleClick}>{statusText}</button>
//     </>
//   );
// }

// function Monitor() {
//   const fps = useStore(gameboyHandle.store, (state) => state.fps);

//   return <div>FPS: {fps}</div>;
// }

function App() {
  const ref = useRef<HTMLCanvasElement>(null);
  const [scale, setScale] = useState(1);
  const [bridge, setBridge] = useState<GameBoyBridge>();

  useKeyboardController({ gameboy: bridge });
  useGamepadController({ gameboy: bridge });

  return (
    <div>
      <canvas
        ref={ref}
        width={RESOLUTION_X * scale}
        height={RESOLUTION_Y * scale}
      />
      <br />
      {/* <Monitor /> */}
      {/* <Controller canvasRef={ref} /> */}
      <span id="fps" />
      <button
        onClick={() => {
          bridge?.play();
        }}
      >
        Play
      </button>
      <button
        onClick={() => {
          bridge?.pause();
        }}
      >
        Pause
      </button>
      <input
        type="file"
        accept=".gb"
        onChange={async (evt) => {
          const file = evt.target.files?.[0];
          if (!file) return;

          const canvas = ref.current!;
          // FIXME: can only transfer once
          const offscreen = canvas.transferControlToOffscreen();

          if (bridge) {
            bridge.install(file, offscreen, scale);
          } else {
            const createdBridge = await GameBoyBridge.create();
            createdBridge.install(file, offscreen, scale);
            setBridge(createdBridge);
          }
        }}
      />
    </div>
  );
}

export { App };
