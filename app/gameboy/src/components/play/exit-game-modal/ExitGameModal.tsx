import { Modal } from "gameboy/components/core/modal";
import { Button } from "gameboy/components/ui/button";
import { ModalCanceledError, ModalOpenedError } from "gameboy/model/error";
import { useImperativeHandle, useRef, useState } from "react";
import type { Ref } from "react";

type IAction = "snapshot" | "no_snapshot" | "cancel";

export interface IExitGameModalRef {
  open: () => Promise<Exclude<IAction, "cancel">>;
}

export function ExitGameModal({ ref }: { ref: Ref<IExitGameModalRef> }) {
  const [open, setOpen] = useState(false);
  const onClose = useRef<(action: IAction) => void>(undefined);

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
            variant="secondary"
            style={{ marginRight: 10 }}
            onClick={() => {
              onClose.current?.("cancel");
            }}
          >
            取消
          </Button>
          <Button
            style={{ marginRight: 10 }}
            onClick={() => {
              onClose.current?.("no_snapshot");
            }}
          >
            不创建
          </Button>
          <Button
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
}
