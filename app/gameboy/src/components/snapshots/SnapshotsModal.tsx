import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetTitle,
} from "gameboy/components/ui/sheet";
import { ModalOpenedError } from "gameboy/model/error";
import { useImperativeHandle, useRef, useState } from "react";
import type { Ref } from "react";

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
    <Sheet
      open={open}
      onOpenChange={(open) => {
        if (!open) {
          onClose.current?.();
        }
      }}
    >
      <SheetContent
        side="right"
        showCloseButton={false}
        className="w-sm gap-0 border-l-0 bg-background text-base shadow-none sm:max-w-sm"
      >
        <SheetTitle className="sr-only" />
        <SheetDescription className="sr-only" />
        {open && <Snapshots {...props.snapshotsProps} />}
      </SheetContent>
    </Sheet>
  );
}
