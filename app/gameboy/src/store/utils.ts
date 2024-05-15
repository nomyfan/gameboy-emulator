import { createStore } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";

export function create<T>(initializer: () => T) {
  return createStore(subscribeWithSelector(immer(initializer)));
}
