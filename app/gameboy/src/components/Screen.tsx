import { forwardRef } from "react";

import { cn } from "../lib/utils";

const SCALE = 2;
const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

const Screen = forwardRef<
  HTMLCanvasElement,
  {
    className?: string;
  }
>(function Screen(props, ref) {
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
    >
      <canvas
        className={cn("bg-white border-black border-[5px]")}
        ref={ref}
        height={RESOLUTION_Y * SCALE}
        width={RESOLUTION_X * SCALE}
      />
    </div>
  );
});

export { Screen };
