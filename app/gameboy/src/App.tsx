import { lazy, Suspense } from "react";
import { useStore } from "zustand";

import { Home } from "./pages/home";
import { store } from "./store";

import "./App.css";

const ExitGameModal = lazy(() => import("./pages/exit-game-modal"));
const PlayModel = lazy(() => import("./pages/play"));
const SnapshotModal = lazy(() => import("./pages/snapshot-modal"));

export function App() {
  const snapshotModalOpen = useStore(store, (st) => st.ui.snapshotModalOpen);
  const playModalOpen = useStore(store, (st) => st.ui.playModalOpen);
  const confirmExitModalOpen = useStore(
    store,
    (st) => st.ui.confirmExitModalOpen,
  );

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
      {confirmExitModalOpen !== undefined && (
        <Suspense>
          <ExitGameModal />
        </Suspense>
      )}
    </>
  );
}
