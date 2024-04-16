import { ReactNode, useMemo } from "react";
import { useStore } from "zustand";

import { Avatar } from "../../components/Avatar";
import { GameList } from "../../components/GameList";
import {
  IconAdd,
  IconDelete,
  IconHistory,
  IconPlay,
  IconSettings,
  IconFullscreen,
  IconFullscreenExit,
} from "../../components/Icons";
import { OperationBar } from "../../components/OperationBar";
import * as fs from "../../fs";
import { useFullscreen } from "../../hooks/useFullscreen";
import { actions, store } from "../../store";

import * as styles from "./Home.css";

export function Home() {
  const selected = useStore(
    store,
    (state) => state.ui.selectedCartridgeId !== undefined,
  );

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
          onClick={(id) => {
            console.log("bar " + id + " clicked");
            if (id === "snapshots") {
              actions.toggleSnapshotsDrawer(true);
            } else if (id === "add") {
              fs.pickFile({ accept: ".gb" }).then((file) => {
                console.log("file", file);
              });
              fs.rootDir().then((root) => fs.createDir(root, "gbos/games"));
            } else if (id === "fullscreen") {
              document.body.requestFullscreen();
            } else if (id === "exit-fullscreen") {
              document.exitFullscreen();
            }
          }}
          items={items}
        />
      </section>
    </main>
  );
}
