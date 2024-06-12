import { useMount } from "gameboy/hooks/useMount";
import { useMemo, useState } from "react";

export function useMountState<T>(
  create: () => T,
  onUnmount: (state: T) => void,
) {
  const state = useState(create)[0];

  const node = useMount(() => {
    return () => {
      onUnmount(state);
    };
  });

  return useMemo(() => [state, node] as const, [state, node]);
}
