import type { CSSProperties, ReactNode } from "react";
import { forwardRef } from "react";

export const SCALE = 2;
const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

const Screen = forwardRef<
  HTMLCanvasElement,
  {
    style?: CSSProperties;
    left?: ReactNode;
    right?: ReactNode;
  }
>(function Screen(props, ref) {
  return (
    <div className="flex-center" style={props.style}>
      {props.left}
      <canvas
        className="shrink-0 bg-white border-5 border-solid border-black"
        ref={ref}
        height={RESOLUTION_Y * SCALE}
        width={RESOLUTION_X * SCALE}
      />
      {props.right}
    </div>
  );
});

export { Screen };
