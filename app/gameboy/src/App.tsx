import { useStore } from "zustand";

import { SnapshotsDrawer } from "./components/snapshots";
import { Home } from "./pages/home";
import { store, actions } from "./store";

export function App() {
  const open = useStore(store, (st) => Boolean(st.ui.snapshotsDrawerOpen));

  return (
    <>
      <Home />
      <SnapshotsDrawer
        open={open}
        onClose={() => actions.toggleSnapshotsDrawer(false)}
      />
    </>
  );
}
