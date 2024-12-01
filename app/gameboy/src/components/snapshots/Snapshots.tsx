import * as ContextMenu from "@radix-ui/react-context-menu";
import { Slot } from "@radix-ui/react-slot";
import { useQuery } from "@tanstack/react-query";
import { cva } from "class-variance-authority";
import { ScaleLoader } from "gameboy/components/core/Spin";
import { useObjectURL } from "gameboy/hooks/useObjectURL";
import type { ISnapshot } from "gameboy/model";
import { storage } from "gameboy/storage/indexdb";
import { useAppStore } from "gameboy/store/app";
import type { ReactNode } from "react";
import { createContext, useContext, useMemo } from "react";

const actionItemVariants = cva(
  "min-w-150px bg-white flex items-center rounded outline-none py-1 px-3 cursor-pointer",
  {
    variants: {
      alert: {
        false: "[&[data-highlighted]]:(bg-primary text-white)",
        true: "text-alert [&[data-highlighted]]:(bg-alert text-white)",
      },
    },
    defaultVariants: {
      alert: false,
    },
  },
);

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

  const url = useObjectURL({ data: cover }, [snapshot.id]);

  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger asChild>
        <div className="flex bg-primary w-full my-2 rounded">
          <img
            src={url}
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
                className={actionItemVariants({
                  alert: it.alert,
                })}
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

  const { data, isLoading, refetch } = useQuery({
    queryKey: [gameId],
    queryFn: async () => {
      if (!gameId) {
        return [];
      }

      const data = await storage.snapshotStore.queryByGameId(gameId);
      data.sort((x, y) => {
        return y.time - x.time;
      });

      return data;
    },
  });

  const renderItems = () => {
    if (!data || isLoading) {
      return (
        <div className="flex justify-center p-3">
          <ScaleLoader />
        </div>
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
        await refetch();
      },
    }),
    [refetch],
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
