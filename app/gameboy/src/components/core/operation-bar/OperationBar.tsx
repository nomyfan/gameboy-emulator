import { cn } from "@callcc/toolkit-js/cn";
import { Separator } from "@radix-ui/react-separator";
import { join } from "gameboy/utils";
import type { CSSProperties, ReactNode } from "react";

export interface IBarItem {
  id: string | number;
  icon: ReactNode;
  alert?: boolean;
  onClick: () => void;
}

export interface IOperationBarProps {
  style?: CSSProperties;
  items: IBarItem[][];
  className?: string;
}

export function OperationBar(props: IOperationBarProps) {
  const renderItems = () => {
    const items = props.items
      .filter((group) => !!group.length)
      .map((group) => {
        const key = group.map((item) => item.id).join("-");
        const items = group.map((item) => {
          return (
            <li
              key={item.id}
              className={cn(
                "bg-white rounded-full p-[6px] inline-flex justify-center items-center mx-1",
                item.alert ? "text-alert" : "text-text",
              )}
              onClick={(evt) => {
                evt.stopPropagation();
                item.onClick();
              }}
            >
              {item.icon}
            </li>
          );
        });
        return <ul key={key}>{items}</ul>;
      });

    return join(items, (_, item) => (
      <Separator
        key={`${item.key}-separator`}
        orientation="vertical"
        className="w-[2px] h-[18px] bg-white mx-[5px]"
      />
    ));
  };
  return (
    <ul className={cn("flex-center", props.className)} style={props.style}>
      {renderItems()}
    </ul>
  );
}
