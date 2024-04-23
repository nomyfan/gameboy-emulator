import { Dialog, DialogPortal, DialogContent } from "@radix-ui/react-dialog";
import { Snapshots } from "gameboy/components/snapshots";
import { actions, useAppStore } from "gameboy/store";

import * as styles from "./SnapshotModal.css";

export function SnapshotsModal() {
  const open = useAppStore((st) => st.dialog.snapshot.open);

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
