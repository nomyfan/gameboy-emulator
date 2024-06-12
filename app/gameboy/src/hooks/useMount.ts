import { createElement, useState } from "react";
import { createPortal } from "react-dom";

export function useMount(callback: () => void | (() => void)) {
  return useState(() => {
    let dispose: void | (() => void);
    const element = document.createElement("div");
    return createPortal(
      createElement("div", {
        ref(element: HTMLDivElement | null) {
          if (element) {
            dispose = callback();
          } else if (dispose) {
            dispose();
          }
        },
      }),
      element,
    );
  })[0];
}
