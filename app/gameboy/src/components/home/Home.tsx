import dayjs from "dayjs";
import { Avatar } from "gameboy/components/core/avatar";
import { GameList } from "gameboy/components/core/game-list";
import { OperationBar } from "gameboy/components/core/operation-bar";
import {
  IconAdd,
  IconDelete,
  IconHistory,
  IconPlay,
  IconFullscreen,
  IconFullscreenExit,
  IconFileDownload,
} from "gameboy/components/icons";
import { useToast } from "gameboy/components/toast/useToast";
import * as fs from "gameboy/fs";
import { useFullscreen } from "gameboy/hooks/useFullscreen";
import { storage } from "gameboy/storage/indexdb";
import { actions, useAppStore } from "gameboy/store";
import { ReactNode, useMemo } from "react";

import * as styles from "./Home.css";

export function Home() {
  const { addToast } = useToast();
  const selected = useAppStore((st) => st.selectedGameId !== undefined);

  const isFullscreen = useFullscreen();

  const items = useMemo(() => {
    const items: { id: string; icon: ReactNode; alert?: boolean }[][] = [];
    if (selected) {
      items.push([
        {
          id: "play",
          icon: <IconPlay />,
        },
        { id: "snapshots", icon: <IconHistory /> },
        {
          id: "delete",
          icon: <IconDelete />,
          alert: true,
        },
        {
          id: "export-backup",
          icon: <IconFileDownload />,
        },
      ]);
    }

    items.push([
      {
        id: "add",
        icon: <IconAdd />,
      },
      isFullscreen
        ? {
            id: "exit-fullscreen",
            icon: <IconFullscreenExit />,
          }
        : {
            id: "fullscreen",
            icon: <IconFullscreen />,
          },
      // {
      //   id: "settings",
      //   icon: <IconSettings />,
      // },
    ]);

    return items;
  }, [selected, isFullscreen]);

  return (
    <main className={styles.home}>
      <section
        className={styles.statusBar}
        onClick={() => {
          actions.selectCartridge();
        }}
      >
        <Avatar fallback="O" />
      </section>

      <GameList className={styles.gameList} />

      <section
        onClick={() => {
          actions.selectCartridge();
        }}
      >
        <OperationBar
          className={styles.operationBar}
          onClick={async (id) => {
            if (id === "snapshots") {
              actions.toggleSnapshotModal(true);
            } else if (id === "add") {
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
              if (files && files.length) {
                const removeToast = addToast("正在导入游戏，请稍候...");
                const packFiles = files.filter((file) =>
                  file.name.endsWith(".gbpack"),
                );
                const cartFiles = files.filter(
                  (file) => !file.name.endsWith(".gbpack"),
                );
                for (const file of cartFiles) {
                  await storage.installGame(file);
                }
                for (const packFile of packFiles) {
                  await storage.importGame(packFile);
                }
                removeToast();
                addToast("导入成功");
                await actions.loadGames();
              }
            } else if (id === "fullscreen") {
              await document.body.requestFullscreen();
            } else if (id === "exit-fullscreen") {
              await document.exitFullscreen();
            } else if (id === "delete") {
              try {
                await actions.openConfirmModal({
                  title: "删除",
                  content: "确认要删除该游戏及其所有存档吗？",
                });
              } catch {
                // Cancelled
                return;
              }
              await actions.deleteSelectedGame();
            } else if (id === "play") {
              actions.openPlayModal();
            } else if (id === "export-backup") {
              const removeToast = addToast("正在导出游戏，请稍候...");
              const { pack, filename } = await actions.exportSelectedGame();
              const url = URL.createObjectURL(pack);
              const timestamp = dayjs().format("YYYYMMDDHHmmss");
              fs.downloadFile(url, `${filename}-${timestamp}.gbpack`);
              URL.revokeObjectURL(url);
              removeToast();
              addToast("导出成功");
            }
          }}
          items={items}
        />
      </section>
    </main>
  );
}
