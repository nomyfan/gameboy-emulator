import { Button } from "gameboy/components/button";
import { FlexBox } from "gameboy/components/flex-box";
import { Modal } from "gameboy/components/modal";
import { actions, store } from "gameboy/store";
import { useStore } from "zustand";

export function ExitGameModal() {
  const exitModalOpen = useStore(store, (st) => st.ui.exitModalOpen);

  return (
    <Modal
      open={exitModalOpen}
      title="结束游戏"
      footer={
        <FlexBox justify="end">
          <Button
            style={{ marginRight: 10 }}
            onClick={() => {
              actions.toggleExitGameModal(false);
            }}
          >
            取消
          </Button>
          <Button
            type="primary"
            style={{ marginRight: 10 }}
            onClick={() => {
              actions.toggleExitGameModal(false, "without_snapshot");
            }}
          >
            不创建
          </Button>
          <Button
            type="primary"
            onClick={() => {
              actions.toggleExitGameModal(false, "with_snapshot");
            }}
          >
            确定
          </Button>
        </FlexBox>
      }
    >
      结束并创建一个存档？
    </Modal>
  );
}
