import { Modal } from "gameboy/components/core/modal";
import { useAppStore, actions } from "gameboy/store";

export function ConfirmModal() {
  const { open, title, content, okText, cancelText } = useAppStore(
    (st) => st.dialog.confirm,
  );

  const handleCancel = () => actions.closeConfirmModal(false);
  const handleOk = () => actions.closeConfirmModal(true);

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
