import { useLayoutEffect, useRef, useState } from "react";
// import { new_gameboy } from "gb_wasm_bindings";

// const worker = new Worker("./worker.ts", { type: "module" });
import Worker1 from "./worker?worker";
const worker = new Worker1();

const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

function App() {
  const ref = useRef<HTMLCanvasElement>(null);
  const [scale, setScale] = useState(2);

  useLayoutEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    // TODO: we can just leverage CSS style to scale or transform to scale
    // ctx.setTransform(2, 0, 0, 2, 0, 0);
    ctx.fillRect(0, 0, RESOLUTION_X, RESOLUTION_Y);
    ctx.fillStyle = "gray";
  }, []);

  return (
    <div>
      {/* <canvas
        id="gb_canvas"
        ref={ref}
        width={RESOLUTION_X}
        height={RESOLUTION_Y}
        style={{ width: RESOLUTION_X * scale, height: RESOLUTION_Y * scale }}
      /> */}
      <br />
      <input
        type="file"
        accept=".gb"
        onChange={(evt) => {
          const file = evt.target.files?.[0];
          if (!file) return;
          // TODO: Move to web workers, but how can we update the canvas efficiently?
          // https://github.com/mdn/dom-examples/tree/main/web-workers/offscreen-canvas-worker
          // new_gameboy(file);
          const canvas = document.getElementById(
            "gb_canvas"
          ) as HTMLCanvasElement;
          let offscreen = canvas.transferControlToOffscreen();
          // new_gameboy(file, offscreen);
          worker.postMessage(
            {
              kind: "play",
              payload: { file, canvas: offscreen },
            },
            [offscreen]
          );
        }}
      />
    </div>
  );
}

export default App;
