import { cn } from "@callcc/toolkit-js/cn";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogPortal,
  DialogTitle,
} from "@radix-ui/react-dialog";
import { VisuallyHidden } from "@radix-ui/react-visually-hidden";
import { ModalOpenedError } from "gameboy/model/error";
import { useImperativeHandle, useRef, useState } from "react";
import type { Ref } from "react";

import styles from "./SnapshotModal.module.css";
import type { ISnapshotsProps } from "./Snapshots";
import { Snapshots } from "./Snapshots";

export interface ISnapshotsModalProps {
  snapshotsProps: ISnapshotsProps;
  ref: Ref<ISnapshotsModalRef>;
}

export interface ISnapshotsModalRef {
  open: () => Promise<void>;
}

export function SnapshotsModal({ ref, ...props }: ISnapshotsModalProps) {
  const [open, setOpen] = useState(false);
  const onClose = useRef<() => void>(undefined);

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
          <VisuallyHidden>
            <DialogTitle />
          </VisuallyHidden>
          <VisuallyHidden>
            <DialogDescription />
          </VisuallyHidden>
          <div
            onClick={() => {
              onClose.current?.();
              onClose.current = undefined;
            }}
            className={cn(
              "fixed top-0 left-0 w-full h-full bg-black/75 backdrop-blur-lg animate-[fade-in_300ms_cubic-bezier(0.16,1,0.3,1)]",
            )}
          >
            <div
              className={cn(
                "fixed top-0 right-0 h-screen w-sm bg-bg",
                styles.drawer,
              )}
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
}
