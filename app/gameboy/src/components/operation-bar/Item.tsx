import { clsx } from "clsx";
import type { ReactNode, HtmlHTMLAttributes } from "react";

import * as styles from "./OperationBar.css";

export function Item(props: {
  icon: ReactNode;
  className?: string;
  onClick?: HtmlHTMLAttributes<HTMLElement>["onClick"];
}) {
  return (
    <div
      className={clsx(styles.barItem, props.className)}
      onClick={props.onClick}
    >
      {props.icon}
    </div>
  );
}
