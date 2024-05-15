import * as PrimitiveToast from "@radix-ui/react-toast";

import * as styles from "./Toast.css";

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
            <PrimitiveToast.Close asChild className={styles.close}>
              <span>关闭</span>
            </PrimitiveToast.Close>
          </PrimitiveToast.Root>
        );
      })}
      <PrimitiveToast.Viewport className={styles.viewport} />
    </PrimitiveToast.Provider>
  );
}
