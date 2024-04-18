import * as Dialog from "@radix-ui/react-dialog";

import { Snapshots } from "./Snapshots";
import * as styles from "./Snapshots.css";

export function SnapshotsDrawer(props: {
  open: boolean;
  onClose?: () => void;
}) {
  return (
    <Dialog.Root open={props.open}>
      <Dialog.Portal>
        <Dialog.Overlay
          onClick={() => props.onClose?.()}
          className={styles.overlay}
        />
        <Dialog.Content style={{ pointerEvents: "auto" }}>
          <div className={styles.drawer}>{props.open && <Snapshots />}</div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
