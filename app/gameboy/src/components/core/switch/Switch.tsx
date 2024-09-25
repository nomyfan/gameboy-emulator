import type { SwitchProps } from "@radix-ui/react-switch";
import { Switch as RadixSwitch, SwitchThumb } from "@radix-ui/react-switch";
import { clsx } from "clsx";

export function Switch(props: SwitchProps) {
  return (
    <RadixSwitch
      {...props}
      className={clsx(
        props.className,
        "w-11 h-6 rounded-full relative block bg-primary/70",
        "[&[data-state=checked]]:bg-primary",
      )}
    >
      <SwitchThumb
        className={clsx(
          "block h-5 w-5 bg-white rounded-full transform-translate-x-0.5 transition-transform transition-duration-200",
          "[&[data-state=checked]]:transform-translate-x-5.5",
        )}
      />
    </RadixSwitch>
  );
}
