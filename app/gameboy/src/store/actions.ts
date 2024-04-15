import { store } from "./state";

export function selectCartridge(id?: string) {
  store.setState((state) => {
    state.games.selectedCartridgeId = id;
  });
}
