import { Button } from "gameboy/components/button";
import { FlexBox } from "gameboy/components/flex-box";
import { Modal } from "gameboy/components/modal";
import { actions, store } from "gameboy/store";
import { useStore } from "zustand";

export function ExitGameModal() {
  const open = useStore(store, (st) => st.ui.confirmExitModalOpen);

  return (
    <Modal
      open={open}
      title="结束游戏"
      footer={
        <FlexBox justify="end">
          <Button
            style={{ marginRight: 10 }}
            onClick={() => {
              actions.closeConfirmExitModal("cancel");
            }}
          >
            取消
          </Button>
          <Button
            type="primary"
            style={{ marginRight: 10 }}
            onClick={() => {
              actions.closeConfirmExitModal("no_snapshot");
            }}
          >
            不创建
          </Button>
          <Button
            type="primary"
            onClick={() => {
              actions.closeConfirmExitModal("snapshot");
            }}
          >
            创建
          </Button>
        </FlexBox>
      }
    >
      需要创建存档吗？
    </Modal>
  );
}
