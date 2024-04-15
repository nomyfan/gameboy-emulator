import { clsx } from "clsx";

import * as styles from "./GameList.css";

export interface IListItemProps {
  src: string;
  selected?: boolean;
  onSelected?: () => void;
}

export function Item(props: IListItemProps) {
  return (
    <div
      className={clsx(
        styles.listItem,
        props.selected && styles.listItemSelected,
      )}
      onClick={() => props.onSelected?.()}
    >
      <img src={props.src} style={{ height: "100%", width: "100%" }} />
    </div>
  );
}
