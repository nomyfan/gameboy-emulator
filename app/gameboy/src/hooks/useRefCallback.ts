import type { DependencyList } from "react";
import { useCallback, useRef } from "react";

/**
 * TODO: we can remove this hook when it's GA.
 * @see https://react.dev/reference/react-dom/components/common#returns
 * @param callback
 * @param deps
 */
export function useRefCallback<T>(
  callback: [T] extends [Exclude<T, null>]
    ? (value: T) => void | (() => void)
    : never,
  deps?: DependencyList,
) {
  const dispose = useRef<() => void>();
  return useCallback(
    (value: T | null) => {
      if (value !== null) {
        dispose.current = callback(value) || undefined;
      } else {
        dispose.current?.();
        dispose.current = undefined;
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    deps ?? [callback],
  );
}
