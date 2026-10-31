"use client";

import {
  CircleCheckIcon,
  InfoIcon,
  Loader2Icon,
  OctagonXIcon,
  TriangleAlertIcon,
} from "lucide-react";
import { Toaster as Sonner, type ToasterProps } from "sonner";

const Toaster = ({ ...props }: ToasterProps) => {
  return (
    <Sonner
      theme="light"
      position="top-center"
      offset={8}
      mobileOffset={8}
      gap={8}
      duration={3000}
      expand
      className="toaster group"
      icons={{
        success: <CircleCheckIcon className="size-4" />,
        info: <InfoIcon className="size-4" />,
        warning: <TriangleAlertIcon className="size-4" />,
        error: <OctagonXIcon className="size-4" />,
        loading: <Loader2Icon className="size-4 animate-spin" />,
      }}
      toastOptions={{
        unstyled: true,
        classNames: {
          toast:
            "right-0 left-0 mx-auto flex w-fit items-center rounded bg-popover py-1 pl-3 text-sm text-popover-foreground shadow-[0_0_2px_rgba(0,0,0,0.25)]",
          cancelButton:
            "mx-1 cursor-pointer rounded px-2 py-1 text-xs font-bold hover:bg-[#e5e5e5]",
        },
      }}
      {...props}
    />
  );
};

export { Toaster };
