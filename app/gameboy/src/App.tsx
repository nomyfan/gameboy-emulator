import { useStore } from "zustand";

import { SnapshotsDrawer } from "./components/snapshots";
import { Home } from "./pages/home";
import { PagePlayModal } from "./pages/play";
import { store, actions } from "./store";

export function App() {
  const drawerOpen = useStore(store, (st) =>
    Boolean(st.ui.snapshotsDrawerOpen),
  );
  const playModalOpen = useStore(store, (st) => st.ui.playModalOpen);

  return (
    <>
      <Home />
      <SnapshotsDrawer
        open={drawerOpen}
        onClose={() => actions.toggleSnapshotsDrawer(false)}
      />
      <PagePlayModal open={playModalOpen} />
    </>
  );
}
