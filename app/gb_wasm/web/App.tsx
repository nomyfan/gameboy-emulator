import { RefObject, useEffect, useRef, useState } from "react";
import { GameBoy } from "./gameboy";
import { useStore } from "zustand";
import { JoypadKey } from "gb_wasm_bindings";
import { fromEvent, map, filter, distinctUntilChanged, merge } from "rxjs";

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

const keysMap: Record<string, JoypadKey> = {
  ArrowRight: JoypadKey.Right,
  ArrowLeft: JoypadKey.Left,
  ArrowUp: JoypadKey.Up,
  ArrowDown: JoypadKey.Down,
  a: JoypadKey.A,
  s: JoypadKey.B,
  Enter: JoypadKey.Start,
  Shift: JoypadKey.Select,
};

function useGameBoyControl() {
  useEffect(() => {
    const isKeyWanted = (key: string) => Object.keys(keysMap).includes(key);

    const keydown$ = fromEvent<KeyboardEvent>(document, "keydown").pipe(
      map((evt) => evt.key)
    );
    const keyup$ = fromEvent<KeyboardEvent>(document, "keyup").pipe(
      map((evt) => evt.key)
    );

    const keys$ = merge(
      keydown$.pipe(
        filter(isKeyWanted),
        map((key) => ({ key, pressed: true }))
      ),
      keyup$.pipe(
        filter(isKeyWanted),
        map((key) => ({ key, pressed: false }))
      )
    );

    const keysSub = keys$
      .pipe(
        distinctUntilChanged(
          (prev, cur) => prev.key === cur.key && prev.pressed === cur.pressed
        )
      )
      .subscribe(({ key, pressed }) => {
        gameboyHandle.changeKey(keysMap[key], pressed);
      });

    return () => {
      keysSub.unsubscribe();
    };
  }, []);
}

function App() {
  const ref = useRef<HTMLCanvasElement>(null);
  const [scale, setScale] = useState(2);

  useGameBoyControl();

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
