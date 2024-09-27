import * as Dialog from "@radix-ui/react-dialog";
import { VisuallyHidden } from "@radix-ui/react-visually-hidden";
import { useAppStore } from "gameboy/store/app";

import { Play } from "./Play";

export function PlayModal() {
  const open = useAppStore((st) => st.dialog.play.open);

  return (
    <Dialog.Root open={open}>
      <Dialog.Portal>
        <Dialog.Content>
          <VisuallyHidden>
            <Dialog.Title />
          </VisuallyHidden>
          <VisuallyHidden>
            <Dialog.Description />
          </VisuallyHidden>
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

// biome-ignore lint/style/noDefaultExport: <explanation>
export default PlayModal;
