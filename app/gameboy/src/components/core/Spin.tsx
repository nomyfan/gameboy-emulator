import { useMemo } from "react";

interface ScaleLoaderProps {
  color?: string;
  loading?: boolean;
  width?: number;
  height?: number;
  margin?: number;
  radius?: number;
}

export function ScaleLoader({
  color = "#000000",
  loading = true,
  width = 4,
  height = 35,
  margin = 2,
  radius = 2,
}: ScaleLoaderProps = {}) {
  const bars = useMemo(() => {
    return Array.from({ length: 5 }).map((_, index) => {
      return (
        <div
          // biome-ignore lint/suspicious/noArrayIndexKey: <explanation>
          key={index}
          className="animate-scale"
          style={{
            width,
            height,
            backgroundColor: color,
            margin: `0 ${margin}px`,
            borderRadius: radius,
            animationDelay: `-${(5 - index) * 0.1}s`,
          }}
        />
      );
    });
  }, [width, height, color, margin, radius]);

  if (!loading) {
    return null;
  }

  return <div className="flex-center h-full">{bars}</div>;
}
