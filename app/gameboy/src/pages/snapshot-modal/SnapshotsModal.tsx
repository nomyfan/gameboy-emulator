import * as Dialog from "@radix-ui/react-dialog";
import { Snapshots } from "gameboy/components/snapshots";
import { actions, store } from "gameboy/store";
import { useStore } from "zustand";

import * as styles from "./SnapshotModal.css";

export function SnapshotsModal() {
  const open = useStore(store, (st) => st.ui.snapshotModalOpen);

  return (
    <Dialog.Root open={open}>
      <Dialog.Portal>
        <Dialog.Overlay
          onClick={() => {
            actions.toggleSnapshotModal(false);
          }}
          className={styles.overlay}
        />
        <Dialog.Content style={{ pointerEvents: "auto" }}>
          <div className={styles.drawer}>{open && <Snapshots />}</div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
