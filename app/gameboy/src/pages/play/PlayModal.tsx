import * as Dialog from "@radix-ui/react-dialog";
import { store } from "gameboy/store";
import { useStore } from "zustand";

import { Play } from "./Play";

export function PlayModal() {
  const playModalOpen = useStore(store, (st) => st.ui.playModalOpen);

  return (
    <Dialog.Root open={playModalOpen}>
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
