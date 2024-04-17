import { ReactNode, useMemo } from "react";
import { useStore } from "zustand";

import { Avatar } from "../../components/avatar";
import { GameList } from "../../components/game-list";
import {
  IconAdd,
  IconDelete,
  IconHistory,
  IconPlay,
  IconSettings,
  IconFullscreen,
  IconFullscreenExit,
} from "../../components/icons";
import { OperationBar } from "../../components/operation-bar";
import * as fs from "../../fs";
import * as storage from "../../fs/storage";
import { useFullscreen } from "../../hooks/useFullscreen";
import { actions, store } from "../../store";

import * as styles from "./PageHome.css";

export function PageHome() {
  const selected = useStore(store, (st) => st.selectedGameId !== undefined);

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
        {
          id: "settings",
          icon: <IconSettings />,
        },
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
        {
          id: "settings",
          icon: <IconSettings />,
        },
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
        <Avatar />
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
              actions.toggleSnapshotsDrawer(true);
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
              await actions.deleteSelectedGame();
            } else if (id === "play") {
              // FIXME: fullscreen modal only
              document.body.requestFullscreen();
              actions.togglePlayModal(true);
            }
          }}
          items={items}
        />
      </section>
    </main>
  );
}
