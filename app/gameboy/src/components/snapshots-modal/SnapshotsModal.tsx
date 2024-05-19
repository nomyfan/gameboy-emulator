import { Dialog, DialogPortal, DialogContent } from "@radix-ui/react-dialog";
import { ModalOpenedError } from "gameboy/model/error";
import { forwardRef, useImperativeHandle, useRef, useState } from "react";

import * as styles from "./SnapshotModal.css";
import type { ISnapshotsProps } from "./Snapshots";
import { Snapshots } from "./Snapshots";

export interface ISnapshotsModalProps {
  snapshotsProps: ISnapshotsProps;
}

export interface ISnapshotsModalRef {
  open: () => Promise<void>;
}

export const SnapshotsModal = forwardRef<
  ISnapshotsModalRef,
  ISnapshotsModalProps
>(function SnapshotsModal(props, ref) {
  const [open, setOpen] = useState(false);
  const onClose = useRef<() => void>();

  useImperativeHandle(ref, () => ({
    open: () => {
      if (open) {
        throw new ModalOpenedError();
      }

      return new Promise<void>((resolve) => {
        onClose.current = () => {
          setOpen(false);
          resolve();
          onClose.current = undefined;
        };
        setOpen(true);
      });
    },
  }));

  return (
    <Dialog open={open}>
      <DialogPortal>
        <DialogContent>
          <div
            onClick={() => {
              onClose.current?.();
              onClose.current = undefined;
            }}
            className={styles.overlay}
          >
            <div
              className={styles.drawer}
              onClick={(evt) => {
                evt.stopPropagation();
              }}
            >
              {open && <Snapshots {...props.snapshotsProps} />}
            </div>
          </div>
        </DialogContent>
      </DialogPortal>
    </Dialog>
  );
});
