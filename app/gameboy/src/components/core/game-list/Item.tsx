import { useRefCallback } from "gameboy/hooks/useRefCallback";
import { cn } from "gameboy/utils/cn";
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
  const refCallback = useRefCallback(
    (element: HTMLImageElement) => {
      if (props.cover) {
        const url = URL.createObjectURL(props.cover);
        element.src = url;
        return () => {
          URL.revokeObjectURL(url);
        };
      }
    },
    [props.cover],
  );

  const children = props.children ?? (
    <figure className="h-full w-full">
      <img className="w-full object-cover" alt={props.name} ref={refCallback} />
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
