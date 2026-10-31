import { toast } from "sonner";

function addToast(message: string, options?: { duration?: number }) {
  const id = toast(message, {
    duration: options?.duration,
    cancel: { label: "关闭", onClick: () => {} },
  });
  return () => {
    toast.dismiss(id);
  };
}

const actions = { addToast };

export function useToast() {
  return actions;
}
