import type { CSSProperties } from "react";
import { forwardRef } from "react";

import * as styles from "./Screen.css";

export const SCALE = 2;
const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

const Screen = forwardRef<
  HTMLCanvasElement,
  {
    style?: CSSProperties;
  }
>(function Screen(props, ref) {
  return (
    <div className={styles.screen} style={props.style}>
      <canvas
        ref={ref}
        height={RESOLUTION_Y * SCALE}
        width={RESOLUTION_X * SCALE}
      />
    </div>
  );
});

export { Screen };
