import { useLayoutEffect, useState } from "react";
import type { DependencyList } from "react";

export function useObjectURL(
  props: {
    data: Blob;
    deferDispose?: boolean;
  },
  deps?: DependencyList,
): string;
export function useObjectURL(
  props: {
    data?: Blob;
    deferDispose?: boolean;
  },
  deps?: DependencyList,
): string | undefined;
export function useObjectURL(
  props: {
    data?: Blob;
    /**
     * @default true
     * Dispose the object URL on next macro task.
     */
    deferDispose?: boolean;
  },
  deps?: DependencyList,
): string | undefined {
  const [url, setUrl] = useState<string | undefined>();

  // biome-ignore lint/correctness/useExhaustiveDependencies:
  useLayoutEffect(() => {
    if (!props.data) return;

    const url = URL.createObjectURL(props.data);
    setUrl(url);
    return () => {
      if (props.deferDispose !== false) {
        setTimeout(() => {
          URL.revokeObjectURL(url);
        });
      } else {
        URL.revokeObjectURL(url);
      }
    };
  }, deps);

  return url;
}
