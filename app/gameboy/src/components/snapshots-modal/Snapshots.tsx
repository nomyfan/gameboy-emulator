import * as ContextMenu from "@radix-ui/react-context-menu";
import { Slot } from "@radix-ui/react-slot";
import { FlexBox } from "gameboy/components/core/flex-box";
import { useRefCallback } from "gameboy/hooks/useRefCallback";
import type { ISnapshot } from "gameboy/model";
import { storage } from "gameboy/storage/indexdb";
import { useAppStore } from "gameboy/store";
import { cn } from "gameboy/utils/cn";
import type { ReactNode } from "react";
import { createContext, useContext, useMemo } from "react";
import ScaleLoader from "react-spinners/ScaleLoader";
import useSWR from "swr";

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

  const refCallback = useRefCallback(
    (element: HTMLImageElement) => {
      const url = URL.createObjectURL(cover);
      element.src = url;
      return () => {
        URL.revokeObjectURL(url);
      };
    },
    [cover],
  );

  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger asChild>
        <div className="flex bg-primary w-full my-2 rounded">
          <img
            ref={refCallback}
            alt={snapshot.name}
            style={{ width: 160 * 0.6, height: 144 * 0.6 }}
            className="grow-0 shrink-0 rounded-l"
          />
          <div className="grow shrink-0 basis-0 p-2 text-sm flex items-center text-white break-all">
            <span>
              {snapshot.name}
              <span className="text-xs">
                （{new Date(snapshot.time).toLocaleString()}）
              </span>
            </span>
          </div>
        </div>
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content className="bg-white rounded text-xs text-text p-1 shadow-[0_10px_38px_-10px_rgba(22,23,24,0.35),0_10px_20px_-15px_rgba(22,23,24,0.2)]">
          {props.menuItems.map((it) => {
            return (
              <ContextMenu.Item
                key={it.label}
                className={cn(
                  "min-w-150px bg-white flex items-center rounded outline-none py-1 px-3 cursor-pointer",
                  !it.alert && "[&[data-highlighted]]:(bg-primary text-white)",
                  it.alert && "text-alert",
                  it.alert && "[&[data-highlighted]]:(bg-alert text-white)",
                )}
                onClick={() => {
                  it.onClick(snapshot, context);
                }}
              >
                <Slot className="w-4 h-4 mr-1">{it.icon ?? <i />}</Slot>
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
    <div className="pt-2 px-2 flex flex-col h-full">
      <h1 className="text-lg font-bold">快照存档</h1>

      <div className="grow shrink-0 basis-0 of-y-auto">
        <SnapshotsContext.Provider value={contextValue}>
          {renderItems()}
        </SnapshotsContext.Provider>
      </div>
    </div>
  );
}
