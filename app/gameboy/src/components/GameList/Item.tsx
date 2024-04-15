import { clsx } from "clsx";

import * as styles from "./GameList.css";

export interface IListItemProps {
  coverURL: string;
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
      onClick={(evt) => {
        if (props.onSelected) {
          evt.stopPropagation();
          props.onSelected();
        }
      }}
    >
      <img src={props.coverURL} style={{ height: "100%", width: "100%" }} />
    </div>
  );
}
