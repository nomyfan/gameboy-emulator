import { clsx } from "clsx";
import { CSSProperties, PropsWithChildren } from "react";

import * as styles from "./GameList.css";

export type IListItemProps = PropsWithChildren<{
  coverURL?: string;
  selected?: boolean;
  onSelected?: () => void;
  style?: CSSProperties;
  className?: string;
}>;

export function Item(props: IListItemProps) {
  const children = props.children ?? (
    <img src={props.coverURL} style={{ height: "100%", width: "100%" }} />
  );

  return (
    <div
      className={clsx(
        styles.listItem,
        props.selected && styles.listItemSelected,
        props.className,
      )}
      style={props.style}
      onClick={(evt) => {
        if (props.onSelected) {
          evt.stopPropagation();
          props.onSelected();
        }
      }}
    >
      {children}
    </div>
  );
}
