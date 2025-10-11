import { cva } from "class-variance-authority";
import type { VariantProps } from "class-variance-authority";
import type { ExcludeNullValue } from "gameboy/types";
import type { ButtonHTMLAttributes } from "react";

const buttonVariants = cva(
  "py-2 px-4 font-medium text-sm rounded-md flex items-center gap-1 disabled:opacity-50 disabled:pointer-events-none",
  {
    variants: {
      variant: {
        primary: "text-white bg-primary",
        danger: "text-white bg-alert",
        default: "text-text bg-white",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

export function Button(
  props: ButtonHTMLAttributes<HTMLButtonElement> & {
    loading?: boolean;
  } & ExcludeNullValue<VariantProps<typeof buttonVariants>>,
) {
  const { variant, disabled, loading, children, ...restProps } = props;

  const loadingOrDisabled = loading || disabled;

  return (
    <button
      {...restProps}
      className={buttonVariants({ variant, className: props.className })}
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
