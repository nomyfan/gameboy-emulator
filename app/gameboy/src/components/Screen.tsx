import clsx from "clsx";
import type { CSSProperties, ReactNode } from "react";
import { forwardRef } from "react";

import * as styles from "./Screen.css";

export const SCALE = 2;
const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

const Screen = forwardRef<
  HTMLCanvasElement,
  {
    className?: string;
    style?: CSSProperties;
    left?: ReactNode;
    right?: ReactNode;
  }
>(function Screen(props, ref) {
  return (
    <div className={clsx(styles.screen, props.className)} style={props.style}>
      {props.left}
      <canvas
        style={{ flexShrink: 0 }}
        ref={ref}
        height={RESOLUTION_Y * SCALE}
        width={RESOLUTION_X * SCALE}
      />
      {props.right}
    </div>
  );
});

export { Screen };
