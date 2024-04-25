import "./App.css";

import { lazy, Suspense } from "react";
import { useRegisterSW } from "virtual:pwa-register/react";

import { useAppStore, actions } from "./store";

const Home = lazy(() => import("./components/home"));
const ExitGameModal = lazy(() => import("./components/exit-game-modal"));
const PlayModel = lazy(() => import("./components/play"));
const SnapshotModal = lazy(() => import("./components/snapshot-modal"));
const ConfirmModal = lazy(() => import("./components/confirm-modal"));

const handleBeforeUnload = (evt: BeforeUnloadEvent) => {
  evt.preventDefault();
};

window.addEventListener("beforeunload", handleBeforeUnload);

export function App() {
  const snapshotModalOpen = useAppStore((st) => st.dialog.snapshot.open);
  const playModalOpen = useAppStore((st) => st.dialog.play.open);
  const exitConfirmModalOpen = useAppStore(
    (st) => st.dialog.exitGameConfirm.open,
  );
  const confirmModalOpen = useAppStore((st) => st.dialog.confirm.open);

  const { updateServiceWorker } = useRegisterSW({
    onNeedRefresh() {
      (async () => {
        try {
          await actions.openConfirmModal({
            title: "有新版本可用！",
            content: "更新将导致当前操作进度丢失，是否立即更新？",
            okText: "立即更新",
            cancelText: "稍后更新",
          });
        } catch {
          // Cancelled
          return;
        }
        window.removeEventListener("beforeunload", handleBeforeUnload);
        await updateServiceWorker(true);
      })();
    },
  });

  return (
    <>
      <Suspense>
        <Home />
      </Suspense>
      {snapshotModalOpen !== undefined && (
        <Suspense>
          <SnapshotModal />
        </Suspense>
      )}
      {playModalOpen !== undefined && (
        <Suspense>
          <PlayModel />
        </Suspense>
      )}
      {exitConfirmModalOpen !== undefined && (
        <Suspense>
          <ExitGameModal />
        </Suspense>
      )}
      {confirmModalOpen !== undefined && (
        <Suspense>
          <ConfirmModal />
        </Suspense>
      )}
    </>
  );
}
