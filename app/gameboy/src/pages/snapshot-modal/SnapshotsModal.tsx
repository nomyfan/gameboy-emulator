import * as Dialog from "@radix-ui/react-dialog";
import { Snapshots } from "gameboy/components/snapshots";
import { actions, store } from "gameboy/store";
import { useStore } from "zustand";

import * as styles from "./SnapshotModal.css";

export function SnapshotsModal() {
  const drawerOpen = useStore(store, (st) => st.ui.snapshotsDrawerOpen);

  return (
    <Dialog.Root open={drawerOpen}>
      <Dialog.Portal>
        <Dialog.Overlay
          onClick={() => {
            actions.toggleSnapshotsDrawer(false);
          }}
          className={styles.overlay}
        />
        <Dialog.Content style={{ pointerEvents: "auto" }}>
          <div className={styles.drawer}>{drawerOpen && <Snapshots />}</div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
