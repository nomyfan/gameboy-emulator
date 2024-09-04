import { cn } from "@callcc/toolkit-js/cn";
import type { ButtonHTMLAttributes } from "react";

export function Button(
  props: Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type"> & {
    type?: "primary" | "normal";
    loading?: boolean;
  },
) {
  const { type, disabled, loading, children, ...restProps } = props;

  const loadingOrDisabled = loading || disabled;

  return (
    <button
      {...restProps}
      className={cn(
        "py-2 px-4 font-medium text-sm rounded-md flex items-center gap-1",
        type === "primary"
          ? "text-white"
          : loadingOrDisabled
            ? "text-text/80"
            : "text-text",
        type === "primary"
          ? loadingOrDisabled
            ? "bg-primary/80"
            : "bg-primary"
          : "bg-white",
        props.className,
      )}
      disabled={loadingOrDisabled}
    >
      {loading && (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="1em"
          height="1em"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="animate-spin"
        >
          <path d="M21 12a9 9 0 1 1-6.219-8.56" />
        </svg>
      )}
      {children}
    </button>
  );
}
