import { clsx } from "clsx";
import { useEffect } from "react";
import ScaleLoader from "react-spinners/ScaleLoader";
import { useStore } from "zustand";

import { store, actions } from "../../store";

import * as styles from "./GameList.css";
import { Item } from "./Item";

export interface IListProps {
  className?: string;
}

export function GameList(props: IListProps) {
  const selectedId = useStore(store, (st) => st.ui.selectedCartridgeId);
  const games = useStore(store, (st) => st.games.cartridges);

  useEffect(() => {
    const start = Date.now();
    actions.loadGames(async () => {
      // Avoid flickering
      await new Promise((resolve) =>
        setTimeout(resolve, Math.max(0, 500 - (Date.now() - start))),
      );
    });
  }, []);

  const renderItems = () => {
    if (!games) {
      return (
        <Item className={styles.placeholderItem}>
          <ScaleLoader />
        </Item>
      );
    }

    if (games.length === 0) {
      // TODO: add guide anchor
      return (
        <Item className={styles.placeholderItem}>
          <span>添加你的第一个游戏吧！</span>
        </Item>
      );
    }

    return games.map(({ id, coverURL, name, path }) => {
      return (
        <Item
          key={id}
          coverURL={coverURL}
          selected={selectedId === id}
          onSelected={() => actions.selectCartridge(id)}
        />
      );
    });
  };

  return (
    <div
      className={clsx(styles.list, props.className)}
      style={{
        justifyContent: !games || games.length === 0 ? "center" : undefined,
      }}
      onClick={() => {
        actions.selectCartridge();
      }}
    >
      {renderItems()}
    </div>
  );
}