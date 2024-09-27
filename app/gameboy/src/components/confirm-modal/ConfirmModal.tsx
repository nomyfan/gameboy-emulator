import { Modal } from "gameboy/components/core/modal";
import { useAppStore } from "gameboy/store/app";
import { closeConfirmModal } from "gameboy/store/app";

export function ConfirmModal() {
  const { open, title, content, okText, cancelText } = useAppStore(
    (st) => st.dialog.confirm,
  );

  const handleCancel = () => closeConfirmModal(false);
  const handleOk = () => closeConfirmModal(true);

  return (
    <Modal
      open={open}
      title={title}
      onClose={handleCancel}
      onCancel={handleCancel}
      onOk={handleOk}
      okText={okText}
      cancelText={cancelText}
    >
      {content}
    </Modal>
  );
}
