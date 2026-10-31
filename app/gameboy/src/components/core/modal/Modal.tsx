import { Button } from "gameboy/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "gameboy/components/ui/dialog";
import type { ReactNode } from "react";

export function Modal(props: {
  open?: boolean;
  defaultOpen?: boolean;
  onClose?: () => void;
  title?: ReactNode;
  children?: ReactNode;
  footer?: ReactNode;
  onOk?: () => void;
  okText?: string;
  onCancel?: () => void;
  cancelText?: string;
  fullscreen?: boolean;
}) {
  const renderFooter = () => {
    if (props.footer) {
      return props.footer;
    }

    return (
      <div className="flex justify-end">
        <Button
          variant="secondary"
          style={{ marginRight: 10 }}
          onClick={() => props.onCancel?.()}
        >
          {props.cancelText || "取消"}
        </Button>
        <Button onClick={() => props.onOk?.()}>{props.okText || "确定"}</Button>
      </div>
    );
  };

  return (
    <Dialog
      open={props.open}
      defaultOpen={props.defaultOpen}
      onOpenChange={(open) => {
        if (!open) {
          props.onClose?.();
        }
      }}
    >
      {props.fullscreen ? (
        <DialogContent
          showCloseButton={false}
          showOverlay={false}
          className="top-0 left-0 block h-screen w-screen max-w-none translate-x-0 translate-y-0 rounded-none bg-transparent p-0 text-base ring-0 sm:max-w-none"
        >
          <DialogTitle className="sr-only" />
          <DialogDescription className="sr-only" />
          {props.children}
        </DialogContent>
      ) : (
        <DialogContent
          showCloseButton={false}
          className="w-auto max-w-lg min-w-xs gap-6 rounded-sm bg-background px-6 py-5 text-base ring-0 sm:max-w-lg"
        >
          <DialogTitle className="text-2xl font-bold">
            {props.title}
          </DialogTitle>
          <DialogDescription className="sr-only" />
          <div className="text-sm font-medium">{props.children}</div>
          {renderFooter()}
        </DialogContent>
      )}
    </Dialog>
  );
}
