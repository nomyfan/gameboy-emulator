import { clsx } from "clsx";
import { ReactNode } from "react";

import { Item } from "./Item";
import * as styles from "./OperationBar.css";

export interface IOperationBarProps<ID extends string | number> {
  className?: string;
  onClick?: (id: ID) => void;
  items: Array<{
    id: ID;
    icon: ReactNode;
    alert?: boolean;
  }>;
}

export function OperationBar<ID extends string | number>(
  props: IOperationBarProps<ID>,
) {
  const renderItems = () => {
    return props.items.map((item) => {
      return (
        <Item
          key={item.id}
          icon={item.icon}
          className={clsx(item.alert && styles.barItemAlert)}
          onClick={() => props.onClick?.(item.id)}
        />
      );
    });
  };
  return (
    <div className={clsx(styles.bar, props.className)}>{renderItems()}</div>
  );
}
