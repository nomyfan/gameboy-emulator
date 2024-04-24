import * as Dialog from "@radix-ui/react-dialog";
import { useAppStore } from "gameboy/store";

import { Play } from "./Play";

export function PlayModal() {
  const open = useAppStore((st) => st.dialog.play.open);

  return (
    <Dialog.Root open={open}>
      <Dialog.Portal>
        <Dialog.Content>
          <Play
            style={{
              position: "fixed",
              top: 0,
              left: 0,
              width: "100vw",
              height: "100vh",
            }}
          />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
