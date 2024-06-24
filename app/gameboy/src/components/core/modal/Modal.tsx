import * as Dialog from "@radix-ui/react-dialog";
import { VisuallyHidden } from "@radix-ui/react-visually-hidden";
import { cn } from "gameboy/utils/cn";
import type { ReactNode } from "react";

import { Button } from "../button";
import { FlexBox } from "../flex-box";

import styles from "./Modal.module.css";

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
      <FlexBox justify="end">
        <Button style={{ marginRight: 10 }} onClick={() => props.onCancel?.()}>
          {props.cancelText || "取消"}
        </Button>
        <Button type="primary" onClick={() => props.onOk?.()}>
          {props.okText || "确定"}
        </Button>
      </FlexBox>
    );
  };

  return (
    <Dialog.Root open={props.open} defaultOpen={props.defaultOpen}>
      <Dialog.Portal>
        {!props.fullscreen && (
          <Dialog.Overlay
            className={cn(
              " fixed top-0 left-0 w-full h-full bg-black/75 backdrop-blur-lg",
              styles.overlay,
            )}
            onClick={() => props.onClose?.()}
          />
        )}
        <Dialog.Content
          className={cn(
            "fixed top-0 left-0 w-screen h-screen",
            props.fullscreen && styles.overlay,
          )}
        >
          <Dialog.Title>
            <VisuallyHidden asChild />
          </Dialog.Title>
          <Dialog.Description>
            <VisuallyHidden asChild />
          </Dialog.Description>
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
