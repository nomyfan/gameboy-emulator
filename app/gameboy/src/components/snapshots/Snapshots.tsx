import * as ContextMenu from "@radix-ui/react-context-menu";
import { FlexBox } from "gameboy/components/flex-box";
import { IconDelete } from "gameboy/components/icons";
import type { ISnapshot } from "gameboy/model";
import { storage } from "gameboy/storage/indexdb";
import { store, actions } from "gameboy/store";
import { cn } from "gameboy/utils/cn";
import type { CSSProperties } from "react";
import { useCallback, useEffect, useState } from "react";
import ScaleLoader from "react-spinners/ScaleLoader";
import useSWR from "swr";
import { useStore } from "zustand";

import * as styles from "./Snapshots.css";

function Item(props: {
  snapshot: ISnapshot;
  className?: string;
  style?: CSSProperties;
  onDelete: (data: ISnapshot) => void;
  onPlay: (data: ISnapshot) => void;
}) {
  const snapshot = props.snapshot;
  const cover = snapshot.cover;

  const [coverURL, setCoverURL] = useState<string>();

  useEffect(() => {
    const url = URL.createObjectURL(cover);
    setCoverURL(url);

    return () => {
      URL.revokeObjectURL(url);
    };
  }, [cover]);

  return (
    <ContextMenu.Root
      onOpenChange={(open) => {
        console.log("__DEBUG__ open", open);
      }}
    >
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
                （{new Date(snapshot.time).toLocaleDateString()}）
              </span>
            </span>
          </div>
        </div>
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content className={styles.menuContent}>
          <ContextMenu.Item
            className={cn(styles.menuItem)}
            onClick={() => {
              props.onPlay(snapshot);
            }}
          >
            <div className={styles.menuItemIcon} />
            进入游戏
          </ContextMenu.Item>
          <ContextMenu.Item
            className={cn(styles.menuItem, styles.menuItemAlert)}
            onClick={() => {
              props.onDelete(snapshot);
            }}
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
  const gameId = useStore(store, (st) => st.selectedGameId);

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

  const handleDeleteSnapshot = useCallback((snapshot: ISnapshot) => {
    storage.snapshotStore.delete(snapshot.id);
  }, []);

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
          onDelete={async (snapshot) => {
            handleDeleteSnapshot(snapshot);
            await mutate();
          }}
          onPlay={async (snapshot) => {
            actions.togglePlayModal(true, {
              gameId: snapshot.gameId,
              data: snapshot.data,
              onClose: () => mutate(),
            });
          }}
        />
      );
    });
  };

  return (
    <div className={styles.snapshotsRoot}>
      <h1 className={styles.header}>存档</h1>

      <div className={styles.itemsContainer}>{renderItems()}</div>
    </div>
  );
}
