import {
  forwardRef,
  useId,
  useImperativeHandle,
  useLayoutEffect,
  useRef,
} from "react";

import { cn } from "../lib/utils";

export const SCALE = 2;
const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

function newScreenCanvas() {
  const canvas = document.createElement("canvas");
  canvas.height = RESOLUTION_Y * SCALE;
  canvas.width = RESOLUTION_X * SCALE;
  canvas.style.backgroundColor = "white";
  canvas.style.borderColor = "black";
  canvas.style.borderWidth = "5px";

  return canvas;
}

export interface IScreenRef {
  newCanvasHandle: () => HTMLCanvasElement;
  scale: number;
}

const Screen = forwardRef<
  IScreenRef,
  {
    className?: string;
  }
>(function Screen(props, ref) {
  const id = useId();
  const containerId = id + "-canvas-container";
  const canvasId = id + "-canvas";

  const initialCanvas = useRef(true);

  useLayoutEffect(() => {
    const container = document.getElementById(containerId) as HTMLDivElement;
    const canvas = newScreenCanvas();
    canvas.id = canvasId;
    container.appendChild(canvas);

    return () => {
      document.getElementById(canvasId)?.remove();
    };
  }, [containerId, canvasId]);

  useImperativeHandle(ref, () => {
    return {
      scale: SCALE,
      newCanvasHandle: () => {
        if (initialCanvas.current) {
          initialCanvas.current = false;
          return document.getElementById(canvasId) as HTMLCanvasElement;
        } else {
          document.getElementById(canvasId)?.remove();

          const container = document.getElementById(containerId) as HTMLElement;
          const canvas = newScreenCanvas();
          canvas.id = canvasId;
          container.appendChild(canvas);

          return canvas;
        }
      },
    };
  });

  return (
    <div
      className={cn(
        "flex justify-center items-center py-5 bg-[#3E3C48]",
        props.className,
      )}
      style={{
        boxShadow:
          "4px 4px 4px rgba(0,0,0,.25),-4px -4px 4px rgba(255,255,255,.25)",
        borderRadius: "0 0 10px 10px",
      }}
      id={containerId}
    />
  );
});

export { Screen };
