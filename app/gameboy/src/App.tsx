import { useStore } from "zustand";

import { Button } from "./components/button";
import { FlexBox } from "./components/flex-box";
import { Modal } from "./components/modal";
import { SnapshotsDrawer } from "./components/snapshots";
import { PageHome } from "./pages/home";
import { PagePlayModal } from "./pages/play";
import { store, actions } from "./store";

export function App() {
  const drawerOpen = useStore(store, (st) =>
    Boolean(st.ui.snapshotsDrawerOpen),
  );
  const playModalOpen = useStore(store, (st) => st.ui.playModalOpen);
  const exitModalOpen = useStore(store, (st) => st.ui.exitModalOpen);

  return (
    <>
      <PageHome />
      <SnapshotsDrawer
        open={drawerOpen}
        onClose={() => actions.toggleSnapshotsDrawer(false)}
      />
      <PagePlayModal open={playModalOpen} />

      <Modal
        open={exitModalOpen}
        title="结束游戏"
        footer={
          <FlexBox justify="end">
            <Button
              style={{ marginRight: 10 }}
              onClick={() => {
                actions.toggleExitModal(false, false);
              }}
            >
              取消
            </Button>
            <Button
              type="primary"
              style={{ marginRight: 10 }}
              onClick={() => {
                actions.toggleExitModal(false, false);
                actions.togglePlayModal(false, false);
              }}
            >
              不创建
            </Button>
            <Button
              type="primary"
              onClick={() => {
                actions.toggleExitModal(false, true);
              }}
            >
              确定
            </Button>
          </FlexBox>
        }
      >
        结束并创建一个存档？
      </Modal>
    </>
  );
}
