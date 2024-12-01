import { cn } from "@callcc/toolkit-js/cn";
import { useObjectURL } from "gameboy/hooks/useObjectURL";
import type { CSSProperties, PropsWithChildren } from "react";

export type IListItemProps = PropsWithChildren<{
  cover?: Blob;
  name?: string;
  selected?: boolean;
  onSelected?: () => void;
  style?: CSSProperties;
  placeholder?: boolean;
}>;

export function Item(props: IListItemProps) {
  const url = useObjectURL({ data: props.cover }, [props.cover]);

  const children = props.children ?? (
    <figure className="h-full w-full">
      <img className="w-full object-cover" alt={props.name} src={url} />
      <figcaption className="text-sm font-medium bg-primary text-white p-1 overflow-hidden whitespace-nowrap text-ellipsis">
        {props.name}
      </figcaption>
    </figure>
  );

  return (
    <div
      className={cn(
        "flex-grow-0 flex-shrink-0 rounded-[1px] border-solid",
        props.placeholder
          ? "flex items-center justify-center w-fit text-primary"
          : "w-48 shadow-[0_4px_4px_rgba(0,0,0,.25)] border-[3px]",
        props.selected ? "border-accent" : "border-primary",
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
