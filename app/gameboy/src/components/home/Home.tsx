import dayjs from "dayjs";
import { Avatar } from "gameboy/components/core/avatar";
import { GameList } from "gameboy/components/core/game-list";
import type { IBarItem } from "gameboy/components/core/operation-bar";
import { OperationBar } from "gameboy/components/core/operation-bar";
import type { IExportModalRef } from "gameboy/components/export/ExportModal";
import { ExportModal } from "gameboy/components/export/ExportModal";
import {
  IconAdd,
  IconDelete,
  IconFileDownload,
  IconFullscreen,
  IconFullscreenExit,
  IconHistory,
  IconPlay,
  IconSettings,
} from "gameboy/components/icons";
import type { ISnapshotsModalRef } from "gameboy/components/snapshots-modal";
import { SnapshotsModal } from "gameboy/components/snapshots-modal";
import { useToast } from "gameboy/components/toast/useToast";
import * as fs from "gameboy/fs";
import { useFullscreen } from "gameboy/hooks/useFullscreen";
import { storage } from "gameboy/storage/indexdb";
import { actions, useAppStore } from "gameboy/store";
import { useMemo, useRef } from "react";

export function Home() {
  const { addToast } = useToast();

  const snapshotsModalRef = useRef<ISnapshotsModalRef>(null);
  const exportModalRef = useRef<IExportModalRef>(null);

  const selected = useAppStore((st) => st.selectedGameId !== undefined);

  const isFullscreen = useFullscreen();

  const items = useMemo(() => {
    const items: IBarItem[][] = [];
    if (selected) {
      items.push([
        {
          id: "play",
          icon: <IconPlay />,
          onClick: () => {
            actions.openPlayModal();
          },
        },
        {
          id: "snapshots",
          icon: <IconHistory />,
          onClick: () => {
            snapshotsModalRef.current?.open();
          },
        },
        {
          id: "delete",
          icon: <IconDelete />,
          alert: true,
          onClick: async () => {
            await actions.openConfirmModal({
              title: "删除",
              content: "确认要删除该游戏及其所有存档吗？",
            });
            await actions.deleteSelectedGame();
          },
        },
        {
          id: "export-backup",
          icon: <IconFileDownload />,
          onClick: async () => {
            await exportModalRef.current?.open();
          },
        },
      ]);
    }

    items.push([
      {
        id: "add",
        icon: <IconAdd />,
        onClick: async () => {
          let files: File[] | null = null;
          try {
            files = await fs
              .pickFile({
                accept: ".gb,.gbc,.gbpack",
                multiple: true,
              })
              .then((files) => (files ? Array.from(files) : null));
          } catch {
            // Cancelled
            return;
          }
          if (files?.length) {
            const removeToast = addToast("正在导入，请稍候...");
            const packFiles = files.filter((file) =>
              file.name.endsWith(".gbpack"),
            );
            const cartFiles = files.filter(
              (file) => !file.name.endsWith(".gbpack"),
            );
            for (const file of cartFiles) {
              await storage.installGame(file);
            }
            for (const pack of packFiles) {
              await storage.importPack(pack);
            }
            removeToast();
            addToast("导入成功");
            await actions.loadGames();
          }
        },
      },
      isFullscreen
        ? {
            id: "exit-fullscreen",
            icon: <IconFullscreenExit />,
            onClick: async () => {
              await document.exitFullscreen();
            },
          }
        : {
            id: "fullscreen",
            icon: <IconFullscreen />,
            onClick: async () => {
              await document.body.requestFullscreen();
            },
          },
      {
        id: "settings",
        icon: <IconSettings />,
        onClick: () => {
          actions.openSettingsModal();
        },
      },
    ]);

    return items;
  }, [selected, isFullscreen, addToast]);

  return (
    <main className="bg-bg h-screen w-screen flex flex-col">
      <section
        className="p-2"
        onClick={() => {
          actions.selectCartridge();
        }}
      >
        <Avatar fallback="O" />
      </section>

      <GameList className="grow shrink-0" />

      <section
        onClick={() => {
          actions.selectCartridge();
        }}
      >
        <OperationBar style={{ padding: "10px 0" }} items={items} />
      </section>

      <SnapshotsModal
        ref={snapshotsModalRef}
        snapshotsProps={{
          actionItems: [
            {
              label: "进入游戏",
              onClick: async (snapshot, { refresh }) => {
                const action = await actions.openPlayModal({
                  gameId: snapshot.gameId,
                  data: snapshot.data,
                });
                if (action === "snapshot") {
                  await refresh();
                }
              },
            },
            {
              label: "导出",
              onClick: async (snapshot, _context) => {
                const removeToast = addToast("正在导出存档，请稍候...");
                const { pack, filename } = await actions.exportSnapshot(
                  snapshot.id,
                  snapshot,
                );
                const url = URL.createObjectURL(pack);
                const timestamp = dayjs().format("YYYYMMDDHHmmss");
                fs.downloadFile(url, `${filename}-${timestamp}.gbpack`);
                URL.revokeObjectURL(url);
                removeToast();
                addToast("导出成功");
              },
            },
            {
              icon: <IconDelete />,
              label: "删除",
              alert: true,
              onClick: async (snapshot, { refresh }) => {
                await actions.openConfirmModal({
                  title: "删除",
                  content: "确认要删除该存档吗？",
                });
                await storage.snapshotStore.delete(snapshot.id);
                await refresh();
              },
            },
          ],
        }}
      />

      <ExportModal ref={exportModalRef} />
    </main>
  );
}
