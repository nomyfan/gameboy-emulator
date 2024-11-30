import { useRegisterSW } from "virtual:pwa-register/react";
import { QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "gameboy/components/toast/ToastProvider";
import { openConfirmModal, useAppStore } from "gameboy/store/app";
import { Suspense, lazy } from "react";
import { queryClient } from "./query";

const LazyHome = lazy(() => import("./components/home"));
const LazyPlayModel = lazy(() => import("./components/play/PlayModal"));
const LazyConfirmModal = lazy(() => import("./components/confirm-modal"));
const LazySettingsModal = lazy(
  () => import("./components/settings/SettingsModal"),
);

const handleBeforeUnload = (evt: BeforeUnloadEvent) => {
  evt.preventDefault();
};

if (process.env.NODE_ENV !== "development") {
  window.addEventListener("beforeunload", handleBeforeUnload);
}

function PlayModal() {
  const open = useAppStore((st) => st.dialog.play.open);
  return open !== undefined ? (
    <Suspense>
      <LazyPlayModel />
    </Suspense>
  ) : null;
}

function ConfirmModal() {
  const open = useAppStore((st) => st.dialog.confirm.open);

  return open !== undefined ? (
    <Suspense>
      <LazyConfirmModal />
    </Suspense>
  ) : null;
}

function SettingsModal() {
  const open = useAppStore((st) => st.dialog.settings.open);

  return open !== undefined ? (
    <Suspense>
      <LazySettingsModal />
    </Suspense>
  ) : null;
}

export function App() {
  const { updateServiceWorker } = useRegisterSW({
    onNeedRefresh() {
      (async () => {
        await openConfirmModal({
          title: "有新版本可用！",
          content: "更新将导致当前操作进度丢失，是否立即更新？",
          okText: "立即更新",
          cancelText: "稍后更新",
        });
        if (process.env.NODE_ENV !== "development") {
          window.removeEventListener("beforeunload", handleBeforeUnload);
        }
        await updateServiceWorker(true);
      })();
    },
  });

  return (
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <Suspense>
          <LazyHome />
        </Suspense>
        <PlayModal />
        <SettingsModal />
        <ConfirmModal />
      </ToastProvider>
    </QueryClientProvider>
  );
}
