import type {
  IToastContextValue,
  IToastItem,
} from "gameboy/components/core/toast";
import { Toast, ToastContext } from "gameboy/components/core/toast";
import { create } from "gameboy/store/utils";
import type { PropsWithChildren } from "react";
import { useState } from "react";
import { useStore } from "zustand";

import { useToast } from "./useToast";

function createStore() {
  return create(() => ({
    toasts: [] as IToastItem[],
  }));
}

function Toasts(props: { storeApi: ReturnType<typeof createStore> }) {
  const { removeToast } = useToast();
  const toasts = useStore(props.storeApi, (st) => st.toasts);

  return (
    <Toast
      toasts={toasts}
      onOpenChange={(id, open) => {
        if (!open) {
          removeToast(id);
        }
      }}
    />
  );
}

export function ToastProvider(props: PropsWithChildren) {
  const [{ storeApi, actions }] = useState(() => {
    const storeApi = createStore();

    let nextId = 0;

    const removeToast: IToastContextValue["removeToast"] = (id) => {
      storeApi.setState((state) => {
        const index = state.toasts.findIndex((it) => it.id === id);
        if (index !== -1) {
          state.toasts.splice(index, 1);
        }
      });
    };

    const clearToasts: IToastContextValue["clearToasts"] = () => {
      storeApi.setState({ toasts: [] });
    };

    const addToast: IToastContextValue["addToast"] = (message, options) => {
      const id = options?.id ?? String(nextId);
      nextId++;
      storeApi.setState((state) => {
        if (state.toasts.findIndex((it) => it.id === id) === -1) {
          state.toasts.push({ id, message, duration: options?.duration });
        }
      });
      return () => {
        removeToast(id);
      };
    };

    return { actions: { addToast, removeToast, clearToasts }, storeApi };
  });

  return (
    <ToastContext.Provider value={actions}>
      {props.children}
      <Toasts storeApi={storeApi} />
    </ToastContext.Provider>
  );
}
