import { clsx } from "clsx";
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

  const renderItems = () => {
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
    <div className={clsx(styles.list, props.className)}>{renderItems()}</div>
  );
}
