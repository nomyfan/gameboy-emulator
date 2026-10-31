import { Modal } from "gameboy/components/core/modal";
import { Settings } from "gameboy/components/settings/Settings";
import { useAppStore } from "gameboy/store/app";

export function SettingsModal() {
  const open = useAppStore((st) => st.dialog.settings.open);

  return (
    <Modal open={open} fullscreen>
      <Settings />
    </Modal>
  );
}

// oxlint-disable-next-line import/no-default-export
export default SettingsModal;
