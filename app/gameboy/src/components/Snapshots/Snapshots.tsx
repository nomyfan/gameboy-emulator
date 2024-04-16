import * as ContextMenu from "@radix-ui/react-context-menu";
import { clsx } from "clsx";
import type { CSSProperties } from "react";

import mockSnapshot from "../../../assets/capture.png";
import { IconDelete } from "../Icons";

import * as styles from "./Snapshots.css";

function Item(props: {
  id: string;
  className?: string;
  style?: CSSProperties;
}) {
  return (
    <ContextMenu.Root
      onOpenChange={(open) => {
        console.log("__DEBUG__ open", open);
      }}
    >
      <ContextMenu.Trigger asChild>
        <div className={styles.item}>
          <img src={mockSnapshot} className={styles.itemImage} />
          <div className={styles.itemDesc}>
            <span>
              存档11111111111111111111111
              <span className={styles.itemSubDesc}>（2020/10/10）</span>
            </span>
          </div>
        </div>
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content className={styles.menuContent}>
          <ContextMenu.Item className={clsx(styles.menuItem)}>
            <div className={styles.menuItemIcon} />
            进入游戏
          </ContextMenu.Item>
          <ContextMenu.Item
            className={clsx(styles.menuItem, styles.menuItemAlert)}
          >
            <IconDelete className={styles.menuItemIcon} />
            删除
          </ContextMenu.Item>
        </ContextMenu.Content>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  );
}

export function Snapshots() {
  const renderItems = () => {
    return Array.from({ length: 20 }).map((_, i) => {
      return <Item key={i} id={i.toString()} />;
    });
  };

  return (
    <div className={styles.snapshotsRoot}>
      <h1 className={styles.header}>存档</h1>

      <div className={styles.itemsContainer}>{renderItems()}</div>
    </div>
  );
}
