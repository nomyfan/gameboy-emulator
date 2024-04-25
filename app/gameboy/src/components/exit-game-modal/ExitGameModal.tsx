import { Button } from "gameboy/components/core/button";
import { FlexBox } from "gameboy/components/core/flex-box";
import { Modal } from "gameboy/components/core/modal";
import { actions, useAppStore } from "gameboy/store";

export function ExitGameModal() {
  const open = useAppStore((st) => st.dialog.exitGameConfirm.open);

  return (
    <Modal
      open={open}
      title="结束游戏"
      footer={
        <FlexBox justify="end">
          <Button
            style={{ marginRight: 10 }}
            onClick={() => {
              actions.closeExitConfirmModal("cancel");
            }}
          >
            取消
          </Button>
          <Button
            type="primary"
            style={{ marginRight: 10 }}
            onClick={() => {
              actions.closeExitConfirmModal("no_snapshot");
            }}
          >
            不创建
          </Button>
          <Button
            type="primary"
            onClick={() => {
              actions.closeExitConfirmModal("snapshot");
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
