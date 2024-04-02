import type { CSSProperties } from "react";

import { cn } from "../lib/utils";
import type { IDirectionButton } from "../types";

function Button(props: {
  className?: string;
  style?: CSSProperties;
  onDown?: () => void;
  onUp?: () => void;
}) {
  return (
    <button
      className={cn("bg-black h-full w-full rounded-[4px]", props.className)}
      style={props.style}
      onMouseDown={() => props.onDown?.()}
      onTouchStart={() => props.onDown?.()}
      onMouseUp={() => props.onUp?.()}
      onTouchEnd={() => props.onUp?.()}
    />
  );
}

function DirectionButton(props: {
  onDown?: (button: IDirectionButton) => void;
  onUp?: (button: IDirectionButton) => void;
}) {
  return (
    <div
      className={cn("grid p-[15px]")}
      style={{
        gridTemplateColumns: "42px 42px 42px",
        gridTemplateRows: "42px 40px 42px",
      }}
    >
      <Button
        key="top"
        className={cn("col-start-2 rounded-b-[0]")}
        style={{
          boxShadow: "-4px -4px 4px rgba(255,255,255,.25)",
        }}
        onDown={() => props.onDown?.("UP")}
        onUp={() => props.onUp?.("UP")}
      />
      <Button
        key="left"
        className={cn("row-start-2 rounded-r-[0]")}
        style={{
          boxShadow:
            "0px 4px 4px rgba(0,0,0,.25),-4px -4px 4px rgba(255,255,255,.25)",
        }}
        onDown={() => props.onDown?.("LEFT")}
        onUp={() => props.onUp?.("LEFT")}
      />
      <div
        key="center"
        className={cn(
          "bg-black row-start-2 col-start-2 flex justify-center items-center",
        )}
      >
        <div
          key="circle"
          className={cn("h-[30px] w-[30px] bg-[#E3E1DD] rounded-full")}
          style={{
            boxShadow:
              "inset -4px -4px 4px rgba(255,255,255,.25), inset 4px 4px 4px rgba(0,0,0,.25)",
          }}
        />
      </div>
      <Button
        key="right"
        className={cn("row-start-2 col-start-3 rounded-l-[0]")}
        style={{
          boxShadow:
            "4px 0px 4px rgba(0,0,0,.25),0px 4px 4px rgba(0,0,0,.25),4px -4px 4px rgba(255,255,255,.25)",
        }}
        onDown={() => props.onDown?.("RIGHT")}
        onUp={() => props.onUp?.("RIGHT")}
      />
      <Button
        key="bottom"
        className={cn("row-start-3 col-start-2 rounded-t-[0]")}
        style={{
          boxShadow:
            "0px 4px 4px rgba(0,0,0,.25),-4px 4px 4px rgba(255,255,255,.25)",
        }}
        onDown={() => props.onDown?.("DOWN")}
        onUp={() => props.onUp?.("DOWN")}
      />
    </div>
  );
}

export { DirectionButton };
