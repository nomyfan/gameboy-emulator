import { Modal } from "gameboy/components/core/modal";
import { useAppStore } from "gameboy/store/app";

import { Play } from "./Play";

export function PlayModal() {
  const open = useAppStore((st) => st.dialog.play.open);

  return (
    <Modal open={open} fullscreen>
      <Play
        style={{
          position: "fixed",
          top: 0,
          left: 0,
          width: "100vw",
          height: "100vh",
        }}
      />
    </Modal>
  );
}

// biome-ignore lint/style/noDefaultExport: <explanation>
export default PlayModal;
