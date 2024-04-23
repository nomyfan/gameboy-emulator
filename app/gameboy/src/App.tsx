import { lazy, Suspense } from "react";
import "./App.css";

import { Home } from "./pages/home";
import { useAppStore } from "./store";

const ExitGameModal = lazy(() => import("./pages/exit-game-modal"));
const PlayModel = lazy(() => import("./pages/play"));
const SnapshotModal = lazy(() => import("./pages/snapshot-modal"));
const ConfirmModal = lazy(() => import("./pages/confirm-modal"));

export function App() {
  const snapshotModalOpen = useAppStore((st) => st.dialog.snapshot.open);
  const playModalOpen = useAppStore((st) => st.dialog.play.open);
  const exitConfirmModalOpen = useAppStore(
    (st) => st.dialog.exitGameConfirm.open,
  );
  const confirmModalOpen = useAppStore((st) => st.dialog.confirm.open);

  return (
    <>
      <Home />
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
