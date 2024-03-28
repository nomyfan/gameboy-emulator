import { JoypadKey } from "gb-wasm";
import { RefObject, useEffect, useRef, useState } from "react";
// import { GameBoy } from "./gameboy";
import { fromEvent, map, filter, distinctUntilChanged, merge } from "rxjs";

import GameBoyWorker from "./gameboy-worker?worker";
import { useGamepadController } from "./gamepad";

const worker = new GameBoyWorker();

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

// function useKeyboardController(props: { gameboy: GameBoy }) {
//   const gameboy = props.gameboy;

//   useEffect(() => {
//     const isKeyWanted = (key: string) => Object.keys(keyMapping).includes(key);

//     const keydown$ = fromEvent<KeyboardEvent>(document, "keydown").pipe(
//       map((evt) => evt.key)
//     );
//     const keyup$ = fromEvent<KeyboardEvent>(document, "keyup").pipe(
//       map((evt) => evt.key)
//     );

//     const keys$ = merge(
//       keydown$.pipe(
//         filter(isKeyWanted),
//         map((key) => ({ key, pressed: true }))
//       ),
//       keyup$.pipe(
//         filter(isKeyWanted),
//         map((key) => ({ key, pressed: false }))
//       )
//     );

//     const keysSub = keys$
//       .pipe(
//         distinctUntilChanged(
//           (prev, cur) => prev.key === cur.key && prev.pressed === cur.pressed
//         )
//       )
//       .subscribe(({ key, pressed }) => {
//         gameboy.changeKey(keyMapping[key], pressed);
//       });

//     return () => {
//       keysSub.unsubscribe();
//     };
//   }, [gameboy]);
// }

function App() {
  const ref = useRef<HTMLCanvasElement>(null);
  const [scale, setScale] = useState(3);

  // useKeyboardController({ gameboy: gameboyHandle });
  // useGamepadController({ gameboy: gameboyHandle });

  useEffect(() => {
    const onmessage = (evt: any) => {
      const data = evt.data;
      if (data.type === "fps") {
        document.getElementById("fps")!.innerText =
          `FPS: ${data.payload.toFixed(0)}`;
      }
    };
    worker.onmessage = onmessage;

    return () => {
      worker.onmessage = null;
    };
  }, []);

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
          worker.postMessage({ type: "play" });
        }}
      >
        Play
      </button>
      <button
        onClick={() => {
          worker.postMessage({ type: "pause" });
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
          // const context = canvas.getContext("2d")!;
          // context.setTransform(scale, 0, 0, scale, 0, 0);

          const raw_buffer = await file.arrayBuffer();
          const buffer = new Uint8ClampedArray(raw_buffer);

          // FIXME: can only transfer once
          const offscreen = canvas.transferControlToOffscreen();

          const audioContext = new AudioContext();
          await audioContext.audioWorklet.addModule(
            new URL("./audio-worklet.js", import.meta.url),
          );
          const gameboyAudioNode = new AudioWorkletNode(
            audioContext,
            "GameBoyAudioProcessor",
            {
              numberOfOutputs: 1,
              outputChannelCount: [2],
            },
          );
          gameboyAudioNode.connect(audioContext.destination);

          const sampleRate = audioContext.sampleRate;
          const audioPort = gameboyAudioNode.port;

          console.log("sample rate", sampleRate);
          worker.postMessage(
            {
              type: "install",
              payload: { buffer, offscreen, sampleRate, audioPort },
            },
            [offscreen, audioPort],
          );

          // gameboyHandle.uninstall();
          // gameboyHandle.install(buffer);
          // gameboyHandle.play(context);
        }}
      />
    </div>
  );
}

export { App };
