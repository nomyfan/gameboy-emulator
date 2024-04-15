import { clsx } from "clsx";
import { ReactNode } from "react";

import * as styles from "./OperationBar.css";

export function Item(props: {
  icon: ReactNode;
  className?: string;
  onClick?: () => void;
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
