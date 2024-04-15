import * as styles from "./App.css";
import { Avatar } from "./components/Avatar";
import { GameList } from "./components/GameList";
import {
  IconAdd,
  IconDelete,
  IconHistory,
  IconPlay,
  IconSettings,
} from "./components/Icons";
import { OperationBar } from "./components/OperationBar";

export function App() {
  return (
    <main className={styles.app}>
      <section className={styles.statusBar}>
        <Avatar />
      </section>

      <GameList className={styles.gameList} />

      <OperationBar
        className={styles.operationBar}
        onClick={(id) => {
          console.log("bar " + id + " clicked");
        }}
        items={[
          {
            icon: <IconPlay />,
            id: "play",
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
        ]}
      />
    </main>
  );
}
