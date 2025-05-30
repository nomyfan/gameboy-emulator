import type { CSSProperties, ReactNode, Ref } from "react";

export const SCALE = 2;
const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

export function Screen({
  ref,
  ...props
}: {
  ref?: Ref<HTMLCanvasElement>;
  style?: CSSProperties;
  left?: ReactNode;
  right?: ReactNode;
}) {
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
}
