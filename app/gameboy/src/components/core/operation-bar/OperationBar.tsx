import { Separator } from "@radix-ui/react-separator";
import { join } from "gameboy/utils";
import { cn } from "gameboy/utils/cn";
import type { ReactNode } from "react";
import { Fragment } from "react";

import { Item } from "./Item";
import * as styles from "./OperationBar.css";

export interface IBarItem {
  id: string | number;
  icon: ReactNode;
  alert?: boolean;
  onClick: () => void;
}

export interface IOperationBarProps {
  className?: string;
  items: IBarItem[][];
}

export function OperationBar(props: IOperationBarProps) {
  const renderItems = () => {
    const items = props.items
      .filter((group) => !!group.length)
      .map((group) => {
        const key = group.map((item) => item.id).join("-");
        const items = group.map((item) => {
          return (
            <Item
              key={item.id}
              icon={item.icon}
              className={cn(item.alert ? styles.barItemAlert : styles.barItem)}
              onClick={(evt) => {
                evt.stopPropagation();
                item.onClick();
              }}
            />
          );
        });
        return <Fragment key={key}>{items}</Fragment>;
      });

    return join(items, (_, item) => (
      <Separator
        key={item.key + "-separator"}
        orientation="vertical"
        className={styles.separator}
      />
    ));
  };
  return <div className={cn(styles.bar, props.className)}>{renderItems()}</div>;
}
