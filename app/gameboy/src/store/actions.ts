import { store } from "./state";

export function selectCartridge(id?: string) {
  store.setState((state) => {
    state.ui.selectedCartridgeId = id;
  });
}

export function toggleSnapshotsDrawer(open?: boolean) {
  store.setState((state) => {
    state.ui.snapshotsDrawerOpen = open ?? !state.ui.snapshotsDrawerOpen;
  });
}
