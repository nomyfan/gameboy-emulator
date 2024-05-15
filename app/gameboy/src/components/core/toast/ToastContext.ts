import { createContext } from "react";

export interface IToastContextValue {
  addToast: (
    message: string,
    options?: {
      duration?: number;
      id?: string;
    },
  ) => () => void;
  removeToast: (id: string) => void;
  clearToasts: () => void;
}

export const ToastContext = createContext<IToastContextValue>({
  addToast: () => () => {},
  removeToast: () => {},
  clearToasts: () => {},
});
