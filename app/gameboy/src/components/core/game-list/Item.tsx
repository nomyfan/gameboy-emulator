import { useMountState } from "gameboy/hooks/useMountState";
import { cn } from "gameboy/utils/cn";
import { CSSProperties, PropsWithChildren } from "react";

export type IListItemProps = PropsWithChildren<{
  cover?: Blob;
  name?: string;
  selected?: boolean;
  onSelected?: () => void;
  style?: CSSProperties;
  placeholder?: boolean;
}>;

export function Item(props: IListItemProps) {
  const [coverURL, node] = useMountState(
    () => (props.cover ? URL.createObjectURL(props.cover) : ""),
    (url) => url && URL.revokeObjectURL(url),
  );

  const children = props.children ?? (
    <figure className="h-full w-full">
      <img className="w-full object-cover" alt={props.name} src={coverURL} />
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
      {node}
      {children}
    </div>
  );
}
