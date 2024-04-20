import { cn } from "gameboy/utils/cn";
import type { ReactNode, HtmlHTMLAttributes } from "react";

import * as styles from "./OperationBar.css";

export function Item(props: {
  icon: ReactNode;
  className?: string;
  onClick?: HtmlHTMLAttributes<HTMLElement>["onClick"];
}) {
  return (
    <div
      className={cn(styles.barItem, props.className)}
      onClick={props.onClick}
    >
      {props.icon}
    </div>
  );
}
