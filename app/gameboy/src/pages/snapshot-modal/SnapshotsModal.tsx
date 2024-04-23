import { Dialog, DialogPortal, DialogContent } from "@radix-ui/react-dialog";
import { Snapshots } from "gameboy/components/snapshots";
import { actions, store } from "gameboy/store";
import { useStore } from "zustand";

import * as styles from "./SnapshotModal.css";

export function SnapshotsModal() {
  const open = useStore(store, (st) => st.ui.snapshotModalOpen);

  return (
    <Dialog open={open}>
      <DialogPortal>
        <DialogContent>
          <div
            onClick={() => {
              actions.toggleSnapshotModal(false);
            }}
            className={styles.overlay}
          >
            <div
              className={styles.drawer}
              onClick={(evt) => {
                evt.stopPropagation();
              }}
            >
              {open && <Snapshots />}
            </div>
          </div>
        </DialogContent>
      </DialogPortal>
    </Dialog>
  );
}
