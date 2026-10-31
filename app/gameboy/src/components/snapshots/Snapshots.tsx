import { useQuery } from "@tanstack/react-query";
import { ScaleLoader } from "gameboy/components/core/Spin";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "gameboy/components/ui/context-menu";
import { useObjectURL } from "gameboy/hooks/useObjectURL";
import type { ISnapshot } from "gameboy/model";
import { storage } from "gameboy/storage/indexdb";
import { useAppStore } from "gameboy/store/app";
import type { ReactNode } from "react";
import { createContext, useContext, useMemo } from "react";

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
    <ContextMenu>
      <ContextMenuTrigger asChild>
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
      </ContextMenuTrigger>
      <ContextMenuContent className="min-w-0 rounded ring-0 shadow-[0_10px_38px_-10px_rgba(22,23,24,0.35),0_10px_20px_-15px_rgba(22,23,24,0.2)]">
        {props.menuItems.map((it) => {
          return (
            <ContextMenuItem
              key={it.label}
              variant={it.alert ? "destructive" : "default"}
              className="min-w-[150px] cursor-pointer gap-1 rounded px-3 py-1 text-xs data-[variant=destructive]:focus:bg-destructive data-[variant=destructive]:focus:text-white"
              onClick={() => {
                it.onClick(snapshot, context);
              }}
            >
              <span className="size-4 *:size-full">{it.icon}</span>
              {it.label}
            </ContextMenuItem>
          );
        })}
      </ContextMenuContent>
    </ContextMenu>
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

      <div className="grow shrink-0 basis-0 overflow-y-auto">
        <SnapshotsContext.Provider value={contextValue}>
          {renderItems()}
        </SnapshotsContext.Provider>
      </div>
    </div>
  );
}
