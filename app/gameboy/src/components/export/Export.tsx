import { useRefCallback } from "@callcc/toolkit-js/react/useRefCallback";
import {
  Popover,
  PopoverContent,
  PopoverPortal,
  PopoverTrigger,
} from "@radix-ui/react-popover";
import { clsx } from "clsx";
import dayjs from "dayjs";
import { ScaleLoader } from "gameboy/components/core/Spin";
import { Button } from "gameboy/components/core/button";
import { Switch } from "gameboy/components/core/switch/Switch";
import { IconCheck } from "gameboy/components/icons";
import { useToast } from "gameboy/components/toast/useToast";
import * as fs from "gameboy/fs";
import type { ISnapshot } from "gameboy/model";
import { storage } from "gameboy/storage/indexdb";
import { after } from "gameboy/utils";
import { useId, useState } from "react";
import useSWR from "swr";
import useSWRMutation from "swr/mutation";
import styles from "./Popover.module.css";

function SnapshotCard(props: {
  data: ISnapshot;
  selected: boolean;
  onSelect: (id: ISnapshot["id"], selected: boolean) => void;
}) {
  const data = props.data;

  const refCallback = useRefCallback(
    (element: HTMLDivElement) => {
      const url = URL.createObjectURL(data.cover);
      element.style.backgroundImage = `url(${url})`;
      return () => {
        URL.revokeObjectURL(url);
      };
    },
    [data.cover],
  );

  return (
    <div
      ref={refCallback}
      className="h-35 bg-primary relative rounded-2"
      style={{
        backgroundSize: "cover",
        backgroundPosition: "center",
      }}
    >
      <div className="absolute left-0 right-0 top-0 bottom-0 flex flex-col justify-between">
        <div className="flex bg-bg/40 backdrop-blur-sm p-0.5 text-white rounded-t-2">
          <IconCheck
            checked={props.selected}
            onClick={() => {
              props.onSelect(data.id, !props.selected);
            }}
          />
        </div>

        <div className="text-xs font-medium bg-bg/40 backdrop-blur-sm p-1 flex items-end rounded-b-2">
          <div className="of-hidden whitespace-nowrap text-ellipsis shrink grow">
            {data.name}
          </div>

          <Popover>
            <PopoverTrigger>
              <div className="i-ic-baseline-info text-5" />
            </PopoverTrigger>

            <PopoverPortal>
              <PopoverContent
                className={clsx(
                  styles.PopoverContent,
                  "bg-white shadow-lg rounded-md p-3 text-sm grid cols-[max-content_auto] gap-x-3",
                )}
              >
                <label>名称</label>
                <span>{data.name}</span>

                <label>时间</label>
                <span>{new Date(data.time).toLocaleString()}</span>
              </PopoverContent>
            </PopoverPortal>
          </Popover>
        </div>
      </div>
    </div>
  );
}

export function Export(props: { gameId: string; onCancel: () => void }) {
  const { gameId } = props;
  const id = useId();
  const { addToast } = useToast();

  const { data: snapshots, isLoading: isSnapshotsLoading } = useSWR(
    [gameId],
    async ([gameId]) => {
      return await after(500, async () => {
        if (!gameId) {
          return [];
        }

        const data = await storage.snapshotStore.queryByGameId(gameId);
        data.sort((x, y) => {
          return y.time - x.time;
        });

        return data;
      });
    },
  );

  const { trigger: exportGame, isMutating } = useSWRMutation(
    `${id}/export`,
    async (
      _,
      {
        arg: { gameId, sav, rom, snapshots },
      }: {
        arg: {
          gameId: string;
          sav?: boolean;
          rom?: boolean;
          snapshots: number[];
        };
      },
    ) => {
      if (!sav && !rom && snapshots.length === 0) {
        addToast("请选择导出内容");
        return;
      }
      await after(300, async () => {
        const removeToast = addToast("正在导出，请稍候...");
        const { pack, filename } = await storage.exportGame(gameId, {
          sav,
          rom,
          snapshots,
        });
        const url = URL.createObjectURL(pack);
        const timestamp = dayjs().format("YYYYMMDDHHmmss");
        fs.downloadFile(url, `${filename}-${timestamp}.gbpack`);
        URL.revokeObjectURL(url);
        removeToast();
        addToast("导出成功");
      });
    },
  );

  const [selectedSnapshotIds, setSelectedSnapshotsIds] = useState<
    ISnapshot["id"][]
  >([]);
  const selectedAll =
    selectedSnapshotIds.length === 0
      ? false
      : selectedSnapshotIds.length === snapshots?.length
        ? true
        : "indeterminate";
  const [rom, setRom] = useState(false);
  const [sav, setSav] = useState(false);

  if (isSnapshotsLoading) {
    return (
      <div className="w-screen h-screen bg-bg p-2 flex-center">
        <ScaleLoader />
      </div>
    );
  }

  return (
    <div className="w-screen h-screen bg-bg p-2">
      <div className="grid cols-[max-content_1fr] rows-[auto_auto_1fr_auto] gap-2 h-full">
        <label
          className="font-semibold text-lg md-2.5 mr-2"
          htmlFor={`${id}ROM`}
        >
          ROM
        </label>
        <Switch
          id={`${id}ROM`}
          checked={rom}
          onCheckedChange={(checked) => setRom(checked)}
        />

        <label
          className="font-semibold text-lg md-2.5 mr-2 row-start-2"
          htmlFor={`${id}SAV`}
        >
          游戏存档
        </label>
        <Switch
          id={`${id}SAV`}
          className="row-start-2"
          checked={sav}
          onCheckedChange={(checked) => setSav(checked)}
        />

        {snapshots?.length !== 0 && (
          <div className="row-start-3 row-end-3 col-start-1 col-end-3 flex flex-col">
            <label className="font-semibold text-lg md-2.5 flex items-center">
              快照存档
              <IconCheck
                checked={selectedAll}
                className={selectedAll === true ? "text-primary" : ""}
                onClick={() => {
                  if (selectedAll === true) {
                    setSelectedSnapshotsIds([]);
                  } else {
                    setSelectedSnapshotsIds(snapshots?.map((s) => s.id) ?? []);
                  }
                }}
              />
            </label>

            <div className="basis-0 grow grid cols-[repeat(auto-fill,minmax(8.75rem,1fr))] auto-rows-max gap-2 of-y-auto">
              {snapshots?.map((s) => {
                return (
                  <SnapshotCard
                    key={s.hash}
                    data={s}
                    selected={selectedSnapshotIds.includes(s.id)}
                    onSelect={(id, selected) => {
                      setSelectedSnapshotsIds((ids) => {
                        if (selected) {
                          return [...ids, id];
                        }
                        return ids.filter((x) => x !== id);
                      });
                    }}
                  />
                );
              })}
            </div>
          </div>
        )}

        <div className="row-start-4 row-end-5 col-span-2 flex flex-row-reverse gap-2">
          <Button
            variant="primary"
            loading={isMutating}
            onClick={async () => {
              await exportGame({
                gameId,
                sav,
                rom,
                snapshots: selectedSnapshotIds ?? [],
              });
            }}
            className="flex items-center gap-1"
          >
            导出
          </Button>
          <Button disabled={isMutating} onClick={props.onCancel}>
            取消
          </Button>
        </div>
      </div>
    </div>
  );
}
