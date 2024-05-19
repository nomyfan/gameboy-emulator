import * as ContextMenu from "@radix-ui/react-context-menu";
import { Slot } from "@radix-ui/react-slot";
import { FlexBox } from "gameboy/components/core/flex-box";
import type { ISnapshot } from "gameboy/model";
import { storage } from "gameboy/storage/indexdb";
import { useAppStore } from "gameboy/store";
import { cn } from "gameboy/utils/cn";
import { createContext, ReactNode, useContext, useMemo } from "react";
import { useEffect, useState } from "react";
import ScaleLoader from "react-spinners/ScaleLoader";
import useSWR from "swr";

import * as styles from "./Snapshots.css";

export interface IActionItem {
  icon?: ReactNode;
  label: string;
  alert?: boolean;
  onClick: (
    snapshot: ISnapshot,
    context: { refresh: () => Promise<void> },
  ) => void;
}

const SnapshotsContext = createContext<{
  refresh: () => Promise<void>;
}>({
  refresh: async () => {},
});

function Item(props: { snapshot: ISnapshot; menuItems: IActionItem[] }) {
  const snapshot = props.snapshot;
  const cover = snapshot.cover;
  const context = useContext(SnapshotsContext);

  const [coverURL, setCoverURL] = useState<string>();

  useEffect(() => {
    const url = URL.createObjectURL(cover);
    setCoverURL(url);

    return () => {
      URL.revokeObjectURL(url);
    };
  }, [cover]);

  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger asChild>
        <div className={styles.item}>
          <img
            alt={snapshot.name}
            src={coverURL}
            className={styles.itemImage}
          />
          <div className={styles.itemDesc}>
            <span>
              {snapshot.name}
              <span className={styles.itemSubDesc}>
                （{new Date(snapshot.time).toLocaleString()}）
              </span>
            </span>
          </div>
        </div>
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content className={styles.menuContent}>
          {props.menuItems.map((it) => {
            return (
              <ContextMenu.Item
                key={it.label}
                className={cn(
                  it.alert ? styles.menuItemAlert : styles.menuItem,
                )}
                onClick={() => {
                  it.onClick(snapshot, context);
                }}
              >
                <Slot className={styles.menuItemIcon}>{it.icon ?? <i />}</Slot>
                {it.label}
              </ContextMenu.Item>
            );
          })}
        </ContextMenu.Content>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  );
}

export interface ISnapshotsProps {
  actionItems: IActionItem[];
}

export function Snapshots(props: ISnapshotsProps) {
  const gameId = useAppStore((st) => st.selectedGameId);

  const { data, isLoading, mutate } = useSWR([gameId], async ([gameId]) => {
    if (!gameId) {
      return [];
    }

    const data = await storage.snapshotStore.queryByGameId(gameId);
    data.sort((x, y) => {
      return y.time - x.time;
    });

    return data;
  });

  const renderItems = () => {
    if (!data || isLoading) {
      return (
        <FlexBox justify="center" style={{ padding: 10 }}>
          <ScaleLoader />
        </FlexBox>
      );
    }
    return data.map((snapshot) => {
      return (
        <Item
          key={snapshot.id}
          snapshot={snapshot}
          menuItems={props.actionItems}
        />
      );
    });
  };

  const contextValue = useMemo(
    () => ({
      refresh: async () => {
        await mutate();
      },
    }),
    [mutate],
  );

  return (
    <div className={styles.snapshotsRoot}>
      <h1 className={styles.header}>存档</h1>

      <div className={styles.itemsContainer}>
        <SnapshotsContext.Provider value={contextValue}>
          {renderItems()}
        </SnapshotsContext.Provider>
      </div>
    </div>
  );
}
