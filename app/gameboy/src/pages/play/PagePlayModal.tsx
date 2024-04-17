import * as Dialog from "@radix-ui/react-dialog";

import { PagePlay } from "./PagePlay";

export function PagePlayModal(props: { open?: boolean }) {
  return (
    <Dialog.Root open={props.open}>
      <Dialog.Portal>
        <Dialog.Content>
          <PagePlay
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
