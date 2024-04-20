import { lazy, Suspense } from "react";
import { useStore } from "zustand";

import { Home } from "./pages/home";
import { store } from "./store";

import "./App.css";

const ExitGameModal = lazy(() => import("./pages/exit-game-modal"));
const PlayModel = lazy(() => import("./pages/play"));
const SnapshotsDrawer = lazy(() => import("./pages/snapshot-modal"));

export function App() {
  const drawerOpen = useStore(store, (st) => st.ui.snapshotsDrawerOpen);
  const playModalOpen = useStore(store, (st) => st.ui.playModalOpen);
  const exitModalOpen = useStore(store, (st) => st.ui.exitModalOpen);

  return (
    <>
      <Home />
      {drawerOpen !== undefined && (
        <Suspense>
          <SnapshotsDrawer />
        </Suspense>
      )}
      {playModalOpen !== undefined && (
        <Suspense>
          <PlayModel />
        </Suspense>
      )}
      {exitModalOpen !== undefined && (
        <Suspense>
          <ExitGameModal />
        </Suspense>
      )}
    </>
  );
}
