import * as Dialog from "@radix-ui/react-dialog";
import { VisuallyHidden } from "@radix-ui/react-visually-hidden";
import { useAppStore } from "gameboy/store";

import { Play } from "./Play";

export function PlayModal() {
  const open = useAppStore((st) => st.dialog.play.open);

  return (
    <Dialog.Root open={open}>
      <Dialog.Portal>
        <Dialog.Content>
          <Dialog.Title>
            <VisuallyHidden asChild />
          </Dialog.Title>
          <Dialog.Description>
            <VisuallyHidden asChild />
          </Dialog.Description>
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

// eslint-disable-next-line import/no-default-export
export default PlayModal;
