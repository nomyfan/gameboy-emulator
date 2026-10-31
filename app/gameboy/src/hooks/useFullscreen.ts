import { useLayoutEffect, useState } from "react";

function detectFullscreen() {
  return (
    !!document.fullscreenElement ||
    // oxlint-disable-next-line typescript/no-explicit-any
    !!(document as any).mozFullScreenElement ||
    // oxlint-disable-next-line typescript/no-explicit-any
    !!(document as any).msFullscreenElement
  );
}

/**
 * Detects if the current document is in fullscreen mode.
 */
export function useFullscreen() {
  const [isFullscreen, setIsFullscreen] = useState(detectFullscreen);

  useLayoutEffect(() => {
    const onFullscreenChange = () => {
      setIsFullscreen(detectFullscreen());
    };

    document.addEventListener("fullscreenchange", onFullscreenChange);

    return () => {
      document.removeEventListener("fullscreenchange", onFullscreenChange);
    };
  }, []);

  return isFullscreen;
}
