import { Modal } from "gameboy/components/core/modal";
import { ModalOpenedError } from "gameboy/model/error";
import { useAppStore } from "gameboy/store";
import { forwardRef, useImperativeHandle, useRef, useState } from "react";

import { Export } from "./Export";

export interface IExportModalRef {
  open: () => Promise<void>;
}

export const ExportModal = forwardRef<IExportModalRef>(
  function ExportModal(_, ref) {
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

    const gameId = useAppStore((st) => st.selectedGameId);

    return (
      <Modal open={open} fullscreen>
        {gameId && (
          <Export gameId={gameId} onCancel={() => onClose.current?.()} />
        )}
      </Modal>
    );
  },
);
// eslint-disable-next-line import/no-default-export
export default ExportModal;
