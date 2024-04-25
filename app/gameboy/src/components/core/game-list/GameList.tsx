import { actions, useAppStore } from "gameboy/store";
import { cn } from "gameboy/utils/cn";
import { useEffect } from "react";
import ScaleLoader from "react-spinners/ScaleLoader";

import { FlexBox } from "../flex-box";

import * as styles from "./GameList.css";
import { Item } from "./Item";

export interface IListProps {
  className?: string;
}

export function GameList(props: IListProps) {
  const selectedId = useAppStore((st) => st.selectedGameId);
  const games = useAppStore((st) => st.games);

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

    return games.map(({ id, coverURL, name }) => {
      return (
        <Item
          key={id}
          coverURL={coverURL}
          name={name}
          selected={selectedId === id}
          onSelected={() => actions.selectCartridge(id)}
        />
      );
    });
  };

  return (
    <FlexBox
      className={cn(styles.list, props.className)}
      align="center"
      justify={!games || games.length === 0 ? "center" : undefined}
      onClick={() => {
        actions.selectCartridge();
      }}
    >
      {renderItems()}
    </FlexBox>
  );
}