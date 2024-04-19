import { clsx } from "clsx";
import { CSSProperties, PropsWithChildren } from "react";

import * as styles from "./GameList.css";

export type IListItemProps = PropsWithChildren<{
  coverURL?: string;
  name?: string;
  selected?: boolean;
  onSelected?: () => void;
  style?: CSSProperties;
  className?: string;
}>;

export function Item(props: IListItemProps) {
  const children = props.children ?? (
    <figure>
      <img alt={props.name} src={props.coverURL} />
      <figcaption>{props.name}</figcaption>
    </figure>
  );

  return (
    <div
      className={clsx(
        props.selected ? styles.listItemSelected : styles.listItem,
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
