import * as Dialog from "@radix-ui/react-dialog";
import { VisuallyHidden } from "@radix-ui/react-visually-hidden";
import { clsx } from "clsx";
import type { ReactNode } from "react";

import { Button } from "../button";

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
        <Button style={{ marginRight: 10 }} onClick={() => props.onCancel?.()}>
          {props.cancelText || "取消"}
        </Button>
        <Button variant="primary" onClick={() => props.onOk?.()}>
          {props.okText || "确定"}
        </Button>
      </div>
    );
  };

  return (
    <Dialog.Root open={props.open} defaultOpen={props.defaultOpen}>
      <Dialog.Portal>
        {!props.fullscreen && (
          <Dialog.Overlay
            className={clsx(
              " fixed top-0 left-0 w-full h-full bg-black/75 backdrop-blur-lg animate-[fade-in_300ms_cubic-bezier(0.16,1,0.3,1)]",
            )}
            onClick={() => props.onClose?.()}
          />
        )}
        <Dialog.Content
          className={clsx(
            "fixed top-0 left-0 w-screen h-screen",
            props.fullscreen &&
              "animate-[fade-in_300ms_cubic-bezier(0.16,1,0.3,1)]",
          )}
        >
          <VisuallyHidden>
            <Dialog.Title />
          </VisuallyHidden>
          <VisuallyHidden>
            <Dialog.Description />
          </VisuallyHidden>
          {props.fullscreen ? (
            props.children
          ) : (
            <div className="max-w-lg min-w-xs absolute-center bg-bg py-5 px-6 rounded">
              <h1 className="mb-6 mt-0 text-6 font-bold">{props.title}</h1>
              <div className="my-6 text-sm font-medium">{props.children}</div>

              <div className="mt-6">{renderFooter()}</div>
            </div>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
