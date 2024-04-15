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
import { actions, store } from "../../store";

import * as styles from "./Home.css";

const barItemsWithSelectedGame = [
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
    id: "add",
    icon: <IconAdd />,
  },
  {
    id: "settings",
    icon: <IconSettings />,
  },
];

const barItemsNormal = [
  {
    id: "add",
    icon: <IconAdd />,
  },
  {
    id: "settings",
    icon: <IconSettings />,
  },
];

export function Home() {
  const selected = useStore(
    store,
    (state) => state.games.selectedCartridgeId !== undefined,
  );

  return (
    <main
      className={styles.home}
      onClick={() => {
        actions.selectCartridge();
      }}
    >
      <section className={styles.statusBar}>
        <Avatar />
      </section>

      <GameList className={styles.gameList} />

      <OperationBar
        className={styles.operationBar}
        onClick={(id) => {
          console.log("bar " + id + " clicked");
        }}
        items={selected ? barItemsWithSelectedGame : barItemsNormal}
      />
    </main>
  );
}
