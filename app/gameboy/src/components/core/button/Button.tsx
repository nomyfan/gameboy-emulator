import { cn } from "gameboy/utils/cn";
import type { HTMLAttributes } from "react";

export function Button(
  props: HTMLAttributes<HTMLButtonElement> & { type?: "primary" | "normal" },
) {
  const { type, children, ...restProps } = props;

  return (
    <button
      {...restProps}
      className={cn(
        "py-2 px-4 font-medium text-sm rounded-md",
        type === "primary" ? "bg-primary text-white" : "bg-white text-text",
        props.className,
      )}
    >
      {children}
    </button>
  );
}
