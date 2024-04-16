import { useStore } from "zustand";

import { Avatar } from "../../components/Avatar";
import { GameList } from "../../components/GameList";
import {
  IconAdd,
  IconDelete,
  IconHistory,
  IconPlay,
  IconSettings,
} from "../../components/Icons";
import { OperationBar } from "../../components/OperationBar";
import * as fs from "../../fs";
import { actions, store } from "../../store";

import * as styles from "./Home.css";

const barItemsWithSelectedGame = [
  [
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
  ],
  [
    {
      id: "add",
      icon: <IconAdd />,
    },
    {
      id: "settings",
      icon: <IconSettings />,
    },
  ],
];

const barItemsNormal = [
  [
    {
      id: "add",
      icon: <IconAdd />,
    },
    {
      id: "settings",
      icon: <IconSettings />,
    },
  ],
];

export function Home() {
  const selected = useStore(
    store,
    (state) => state.ui.selectedCartridgeId !== undefined,
  );

  return (
    <main className={styles.home}>
      <section className={styles.statusBar}>
        <Avatar />
      </section>

      <GameList className={styles.gameList} />

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
          }
        }}
        items={selected ? barItemsWithSelectedGame : barItemsNormal}
      />
    </main>
  );
}
