import { useStore } from "zustand";

import { SnapshotsDrawer } from "./components/snapshots";
import { PageHome } from "./pages/home";
import { PagePlayModal } from "./pages/play";
import { store, actions } from "./store";

export function App() {
  const drawerOpen = useStore(store, (st) =>
    Boolean(st.ui.snapshotsDrawerOpen),
  );
  const playModalOpen = useStore(store, (st) => st.ui.playModalOpen);

  return (
    <>
      <PageHome />
      <SnapshotsDrawer
        open={drawerOpen}
        onClose={() => actions.toggleSnapshotsDrawer(false)}
      />
      <PagePlayModal open={playModalOpen} />
    </>
  );
}
