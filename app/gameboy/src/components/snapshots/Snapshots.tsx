import * as ContextMenu from "@radix-ui/react-context-menu";
import { clsx } from "clsx";
import { CSSProperties, useCallback, useEffect, useState } from "react";
import ScaleLoader from "react-spinners/ScaleLoader";
import useSWR from "swr";
import { useStore } from "zustand";

import type { ISnapshot } from "../../model";
import { storage } from "../../storage/indexdb";
import { store, actions } from "../../store";
import { FlexBox } from "../flex-box";
import { IconDelete } from "../icons";

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
            className={clsx(styles.menuItem)}
            onClick={() => {
              props.onPlay(snapshot);
            }}
          >
            <div className={styles.menuItemIcon} />
            进入游戏
          </ContextMenu.Item>
          <ContextMenu.Item
            className={clsx(styles.menuItem, styles.menuItemAlert)}
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

// TODO: notify this component to revalidate snapshot list
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
