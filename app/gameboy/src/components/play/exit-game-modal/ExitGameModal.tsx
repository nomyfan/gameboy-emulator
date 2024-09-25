import { Button } from "gameboy/components/core/button";
import { Modal } from "gameboy/components/core/modal";
import { ModalCanceledError, ModalOpenedError } from "gameboy/model/error";
import { forwardRef, useImperativeHandle, useRef, useState } from "react";

type IAction = "snapshot" | "no_snapshot" | "cancel";

export interface IExitGameModalRef {
  open: () => Promise<Exclude<IAction, "cancel">>;
}

export const ExitGameModal = forwardRef<IExitGameModalRef, unknown>(
  function ExitGameModal(props, ref) {
    const [open, setOpen] = useState(false);
    const onClose = useRef<(action: IAction) => void>();

    useImperativeHandle(ref, () => ({
      open: () => {
        if (open) {
          return Promise.reject(new ModalOpenedError());
        }
        return new Promise<Exclude<IAction, "cancel">>((resolve, reject) => {
          onClose.current = (action) => {
            setOpen(false);
            if (action === "cancel") {
              reject(new ModalCanceledError());
            } else {
              resolve(action);
            }
            onClose.current = undefined;
          };
          setOpen(true);
        });
      },
    }));

    return (
      <Modal
        open={open}
        title="结束游戏"
        footer={
          <div className="flex justify-end">
            <Button
              style={{ marginRight: 10 }}
              onClick={() => {
                onClose.current?.("cancel");
              }}
            >
              取消
            </Button>
            <Button
              variant="primary"
              style={{ marginRight: 10 }}
              onClick={() => {
                onClose.current?.("no_snapshot");
              }}
            >
              不创建
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                onClose.current?.("snapshot");
              }}
            >
              创建
            </Button>
          </div>
        }
      >
        需要创建存档吗？
      </Modal>
    );
  },
);
