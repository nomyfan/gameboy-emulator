import { JoypadKey } from "gb-wasm";
import { useEffect, useRef, useState } from "react";
import { fromEvent, map, filter, distinctUntilChanged, merge } from "rxjs";

import { GameBoyBridge } from "./gameboy-worker-bridge";
import { useGamepadController } from "./gamepad";

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

const keyMapping: Record<string, JoypadKey> = {
  ArrowRight: JoypadKey.Right,
  ArrowLeft: JoypadKey.Left,
  ArrowUp: JoypadKey.Up,
  ArrowDown: JoypadKey.Down,
  a: JoypadKey.A,
  s: JoypadKey.B,
  Enter: JoypadKey.Start,
  Shift: JoypadKey.Select,
};

function useKeyboardController(props: { gameboy: GameBoyBridge | undefined }) {
  const gameboy = props.gameboy;

  useEffect(() => {
    if (!gameboy) return;

    const isKeyWanted = (key: string) => Object.keys(keyMapping).includes(key);

    const keydown$ = fromEvent<KeyboardEvent>(document, "keydown").pipe(
      map((evt) => evt.key),
    );
    const keyup$ = fromEvent<KeyboardEvent>(document, "keyup").pipe(
      map((evt) => evt.key),
    );

    const keys$ = merge(
      keydown$.pipe(
        filter(isKeyWanted),
        map((key) => ({ key, pressed: true })),
      ),
      keyup$.pipe(
        filter(isKeyWanted),
        map((key) => ({ key, pressed: false })),
      ),
    );

    let state = 0;
    const keysSub = keys$
      .pipe(
        distinctUntilChanged(
          (prev, cur) => prev.key === cur.key && prev.pressed === cur.pressed,
        ),
      )
      .subscribe(({ key, pressed }) => {
        if (pressed) {
          state |= keyMapping[key];
        } else {
          state &= ~keyMapping[key];
        }
        gameboy.changeKeyState(state);
      });

    return () => {
      keysSub.unsubscribe();
    };
  }, [gameboy]);
}

function App() {
  const ref = useRef<HTMLCanvasElement>(null);
  const [scale, setScale] = useState(3);
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
      <button
        onClick={() => {
          bridge?.terminate();
        }}
      >
        Terminate
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
