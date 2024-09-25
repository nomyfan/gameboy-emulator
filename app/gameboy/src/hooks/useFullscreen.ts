import { useLayoutEffect, useState } from "react";

function detectFullscreen() {
  return (
    !!document.fullscreenElement ||
    // biome-ignore lint/suspicious/noExplicitAny: <explanation>
    !!(document as any).mozFullScreenElement ||
    // biome-ignore lint/suspicious/noExplicitAny: <explanation>
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
