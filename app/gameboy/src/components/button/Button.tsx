import { cn } from "gameboy/utils/cn";
import type { HTMLAttributes } from "react";

import * as styles from "./Button.css";

export function Button(
  props: HTMLAttributes<HTMLButtonElement> & { type?: "primary" | "normal" },
) {
  const { type, children, ...restProps } = props;

  return (
    <button
      {...restProps}
      className={cn(
        type === "primary" ? styles.buttonPrimary : styles.button,
        props.className,
      )}
    >
      {children}
    </button>
  );
}
