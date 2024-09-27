import { clsx } from "clsx";
import { ScaleLoader } from "gameboy/components/core/Spin";
import { selectCartridge, useAppStore } from "gameboy/store/app";
import { gameStore, loadGames } from "gameboy/store/game";
import { useEffect } from "react";
import { useStore } from "zustand";
import { Item } from "./Item";

export interface IListProps {
  className?: string;
}

export function GameList(props: IListProps) {
  const selectedId = useAppStore((st) => st.selectedGameId);
  const games = useStore(gameStore, (st) => st.games);

  useEffect(() => {
    const start = Date.now();
    loadGames(async () => {
      // Avoid flickering
      await new Promise((resolve) =>
        setTimeout(resolve, Math.max(0, 500 - (Date.now() - start))),
      );
    });
  }, []);

  const renderItems = () => {
    if (!games) {
      return (
        <Item placeholder>
          <ScaleLoader />
        </Item>
      );
    }

    if (games.length === 0) {
      // TODO: add guide anchor
      return (
        <Item placeholder>
          <span>添加你的第一个游戏吧！</span>
        </Item>
      );
    }

    return games.map(({ id, cover, name }) => {
      return (
        <Item
          key={id}
          cover={cover}
          name={name}
          selected={selectedId === id}
          onSelected={() => selectCartridge(id)}
        />
      );
    });
  };

  return (
    <div
      className={clsx(
        "flex items-center gap-2.5 px-2.5 overflow-x-auto",
        (!games || games.length === 0) && "justify-center",
        props.className,
      )}
      onClick={() => {
        selectCartridge();
      }}
    >
      {renderItems()}
    </div>
  );
}
