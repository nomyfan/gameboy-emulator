import { useContext } from "react";

import { ToastContext } from "../core/toast";

export function useToast() {
  return useContext(ToastContext);
}
