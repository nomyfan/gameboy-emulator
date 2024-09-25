import * as RadixAvatar from "@radix-ui/react-avatar";
import { clsx } from "clsx";
import type { ReactNode } from "react";

export function Avatar(props: { src?: string; fallback?: ReactNode }) {
  return (
    <RadixAvatar.Root
      className={clsx(
        "h-10 w-10 block border-solid border-white border-2 rounded-full shadow-[0_4px_4px_rgba(0,0,0,.25)]",
      )}
    >
      <RadixAvatar.Image
        className="h-full w-full object-cover rounded-full"
        src={props.src}
      />
      <RadixAvatar.Fallback asChild>
        <div className="flex justify-center items-center h-full w-full text-sm bg-primary text-white rounded-full">
          {props.fallback}
        </div>
      </RadixAvatar.Fallback>
    </RadixAvatar.Root>
  );
}
