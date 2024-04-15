import { clsx } from "clsx";
import { useState } from "react";

import mockGame1 from "./assets/game1.jpeg";
import mockGame2 from "./assets/game2.png";
import mockGame3 from "./assets/game3.jpeg";
import mockGame4 from "./assets/game4.jpeg";
import mockGame5 from "./assets/game5.jpeg";
import * as styles from "./GameList.css";
import { Item } from "./Item";

const mockImages = [mockGame1, mockGame2, mockGame3, mockGame4, mockGame5];

export interface IListProps {
  className?: string;
}

export function GameList(props: IListProps) {
  // TODO: data source
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const renderItems = () => {
    return mockImages.map((src) => {
      return (
        <Item
          key={src}
          src={src}
          selected={selectedId === src}
          onSelected={() => setSelectedId(src)}
        />
      );
    });
  };

  return (
    <div className={clsx(styles.list, props.className)}>{renderItems()}</div>
  );
}
