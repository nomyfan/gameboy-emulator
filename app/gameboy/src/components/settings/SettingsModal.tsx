import { Dialog, DialogPortal, DialogContent } from "@radix-ui/react-dialog";
import { Settings } from "gameboy/components/settings/Settings";
import { useAppStore } from "gameboy/store";

export function SettingsModal() {
  const open = useAppStore((st) => st.dialog.settings.open);

  return (
    <Dialog open={open}>
      <DialogPortal>
        <DialogContent>
          <div
            style={{ position: "fixed", left: 0, top: 0, right: 0, bottom: 0 }}
          >
            <Settings />
          </div>
        </DialogContent>
      </DialogPortal>
    </Dialog>
  );
}

// eslint-disable-next-line import/no-default-export
export default SettingsModal;
