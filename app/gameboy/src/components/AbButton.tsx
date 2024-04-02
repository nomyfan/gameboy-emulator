import type { ReactNode } from "react";

import { cn } from "../lib/utils";
import type { IAbButton } from "../types";

function Button(props: {
  className?: string;
  onDown?: () => void;
  onUp?: () => void;
}) {
  return (
    <button
      className={cn(
        "bg-[#9B0757] h-[50px] w-[50px] rounded-full",
        props.className,
      )}
      style={{
        boxShadow: "3px 3px 4px rgba(0,0,0,.25)",
      }}
      onMouseDown={() => props.onDown?.()}
      onTouchStart={() => props.onDown?.()}
      onMouseUp={() => props.onUp?.()}
      onTouchEnd={() => props.onUp?.()}
    ></button>
  );
}

function ButtonLabel(props: { className?: string; children?: ReactNode }) {
  return (
    <label
      className={cn("w-[45px]", props.className)}
      style={{
        textShadow:
          "-2px -2px 4px rgba(255,255,255,.25),3px 3px 4px rgba(0,0,0,.25)",
      }}
    >
      {props.children}
    </label>
  );
}

function AbButton(props: {
  onDown?: (button: IAbButton) => void;
  onUp?: (button: IAbButton) => void;
}) {
  return (
    <div
      className="relative"
      style={{
        transform: "rotate(-25deg) translateY(-12px)",
      }}
    >
      <div className={cn("w-fit flex rounded-[50px] py-[10px] px-[15px]")}>
        <Button
          className={cn("mr-[20px] ")}
          onDown={() => props.onDown?.("B")}
          onUp={() => props.onUp?.("B")}
        />
        <Button
          onDown={() => props.onDown?.("A")}
          onUp={() => props.onUp?.("A")}
        />
      </div>

      <div
        className={cn(
          "flex justify-between px-[15px] text-center font-semibold w-full absolute ml-[5px]",
        )}
      >
        <ButtonLabel>B</ButtonLabel>
        <ButtonLabel>A</ButtonLabel>
      </div>
    </div>
  );
}

export { AbButton };
