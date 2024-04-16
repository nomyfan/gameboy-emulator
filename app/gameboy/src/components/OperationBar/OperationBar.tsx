import { Separator } from "@radix-ui/react-separator";
import { clsx } from "clsx";
import type { ReactNode } from "react";
import { Fragment } from "react";

import { Item } from "./Item";
import * as styles from "./OperationBar.css";

export interface IOperationBarProps<ID extends string | number> {
  className?: string;
  onClick?: (id: ID) => void;
  items: Array<
    Array<{
      id: ID;
      icon: ReactNode;
      alert?: boolean;
    }>
  >;
}

export function OperationBar<ID extends string | number>(
  props: IOperationBarProps<ID>,
) {
  const renderItems = () => {
    const nodes: ReactNode[] = [];

    const groups = props.items.filter((it) => !!it.length);
    for (let i = 0; i < groups.length; i++) {
      const groupKey = props.items[i].map((item) => item.id).join("-");
      const group = props.items[i].map((item) => {
        return (
          <Item
            key={item.id}
            icon={item.icon}
            className={clsx(item.alert && styles.barItemAlert)}
            onClick={() => props.onClick?.(item.id)}
          />
        );
      });
      nodes.push(<Fragment key={groupKey}>{group}</Fragment>);
      if (i !== groups.length - 1) {
        nodes.push(
          <Separator
            key={groupKey + "-separator"}
            orientation="vertical"
            className={styles.separator}
          />,
        );
      }
    }

    return nodes;
  };
  return (
    <div className={clsx(styles.bar, props.className)}>{renderItems()}</div>
  );
}
