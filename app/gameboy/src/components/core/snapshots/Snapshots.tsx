import * as ContextMenu from "@radix-ui/react-context-menu";
import { IconDelete } from "gameboy/components/icons";
import type { ISnapshot } from "gameboy/model";
import { storage } from "gameboy/storage/indexdb";
import { actions, useAppStore } from "gameboy/store";
import { cn } from "gameboy/utils/cn";
import type { CSSProperties } from "react";
import { useEffect, useState } from "react";
import ScaleLoader from "react-spinners/ScaleLoader";
import useSWR from "swr";

import { FlexBox } from "../flex-box";

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
          onDelete={async (snapshot) => {
            try {
              await actions.openConfirmModal({
                title: "删除",
                content: "确认要删除该存档吗？",
              });
            } catch {
              // Cancelled
              return;
            }
            await storage.snapshotStore.delete(snapshot.id);
            await mutate();
          }}
          onPlay={async (snapshot) => {
            actions
              .openPlayModal({
                gameId: snapshot.gameId,
                data: snapshot.data,
              })
              .then((action) => {
                if (action === "snapshot") {
                  mutate();
                }
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
