import * as Dialog from "@radix-ui/react-dialog";
import { clsx } from "clsx";
import type { ReactNode } from "react";

import { Button } from "../button";
import { FlexBox } from "../flex-box";

import * as styles from "./Modal.css";

export function Modal(props: {
  open?: boolean;
  defaultOpen?: boolean;
  onClose?: () => void;
  title?: ReactNode;
  children?: ReactNode;
  footer?: ReactNode;
  onOk?: () => void;
  onCancel?: () => void;
}) {
  const renderFooter = () => {
    if (props.footer) {
      return props.footer;
    }

    return (
      <FlexBox justify="end">
        <Button style={{ marginRight: 10 }}>取消</Button>
        <Button type="primary">确定</Button>
      </FlexBox>
    );
  };

  return (
    <Dialog.Root open={props.open} defaultOpen={props.defaultOpen}>
      <Dialog.Portal>
        <Dialog.Overlay
          className={styles.overlay}
          onClick={() => props.onClose?.()}
        />
        <Dialog.Content className={styles.content}>
          <div className={styles.container}>
            <h1 className={styles.title}>{props.title}</h1>
            <div className={styles.description}>{props.children}</div>

            <div className={styles.footer}>{renderFooter()}</div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
