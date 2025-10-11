import { cn } from "@callcc/toolkit-js/cn";
import * as PrimitiveToast from "@radix-ui/react-toast";
import { ToastViewport } from "@radix-ui/react-toast";

import styles from "./Toast.module.css";

export interface IToastItem {
  id: string;
  message: string;
  duration?: number;
}

export function Toast(props: {
  toasts: Array<IToastItem>;
  onOpenChange: (id: string, open: boolean) => void;
}) {
  return (
    <PrimitiveToast.Provider swipeDirection="right">
      {props.toasts.map((it) => {
        return (
          <PrimitiveToast.Root
            key={it.id}
            defaultOpen
            duration={it.duration ?? 3000}
            onOpenChange={(open) => {
              props.onOpenChange(it.id, open);
            }}
          >
            <PrimitiveToast.Description>
              {it.message}
            </PrimitiveToast.Description>
            <PrimitiveToast.Close
              asChild
              className="rounded py-1 px-2 mx-1 text-xs decoration-dashed font-bold hover:bg-[#e5e5e5]"
            >
              <span>关闭</span>
            </PrimitiveToast.Close>
          </PrimitiveToast.Root>
        );
      })}
      <ToastViewport
        className={cn(
          "fixed left-0 text-sm top-0 right-0 m-auto w-fit list-none outline-0 z-36 py-2 px-4 flex flex-col-reverse gap-2",
          "[&>[data-state=open]]:bg-white [&>[data-state=open]]:flex [&>[data-state=open]]:items-center [&>[data-state=open]]:py-1 [&>[data-state=open]]:pl-3 [&>[data-state=open]]:rounded [&>[data-state=open]]:shadow-[0_0_2px_rgba(0,0,0,0.25)]",
          styles.viewport,
        )}
      />
    </PrimitiveToast.Provider>
  );
}
