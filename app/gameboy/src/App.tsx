import { lazy, Suspense } from "react";
import "./App.css";

import { useAppStore } from "./store";

const Home = lazy(() => import("./components/home"));
const ExitGameModal = lazy(() => import("./components/exit-game-modal"));
const PlayModel = lazy(() => import("./components/play"));
const SnapshotModal = lazy(() => import("./components/snapshot-modal"));
const ConfirmModal = lazy(() => import("./components/confirm-modal"));

export function App() {
  const snapshotModalOpen = useAppStore((st) => st.dialog.snapshot.open);
  const playModalOpen = useAppStore((st) => st.dialog.play.open);
  const exitConfirmModalOpen = useAppStore(
    (st) => st.dialog.exitGameConfirm.open,
  );
  const confirmModalOpen = useAppStore((st) => st.dialog.confirm.open);

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
