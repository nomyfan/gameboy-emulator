import { Avatar } from "gameboy/components/avatar";
import { GameList } from "gameboy/components/game-list";
import {
  IconAdd,
  IconDelete,
  IconHistory,
  IconPlay,
  IconFullscreen,
  IconFullscreenExit,
} from "gameboy/components/icons";
import { OperationBar } from "gameboy/components/operation-bar";
import * as fs from "gameboy/fs";
import { useFullscreen } from "gameboy/hooks/useFullscreen";
import { storage } from "gameboy/storage/indexdb";
import { actions, useAppStore } from "gameboy/store";
import { ReactNode, useMemo } from "react";

import * as styles from "./Home.css";

export function Home() {
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
      ]);
    }

    if (isFullscreen) {
      items.push([
        {
          id: "add",
          icon: <IconAdd />,
        },
        {
          id: "exit-fullscreen",
          icon: <IconFullscreenExit />,
        },
        // {
        //   id: "settings",
        //   icon: <IconSettings />,
        // },
      ]);
    } else {
      items.push([
        {
          id: "add",
          icon: <IconAdd />,
        },
        {
          id: "fullscreen",
          icon: <IconFullscreen />,
        },
        // {
        //   id: "settings",
        //   icon: <IconSettings />,
        // },
      ]);
    }

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
            console.log("bar " + id + " clicked");
            if (id === "snapshots") {
              actions.toggleSnapshotModal(true);
            } else if (id === "add") {
              const file = await fs.pickFile({ accept: ".gb" });
              if (file) {
                await storage.installGame(file);
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
            }
          }}
          items={items}
        />
      </section>
    </main>
  );
}
