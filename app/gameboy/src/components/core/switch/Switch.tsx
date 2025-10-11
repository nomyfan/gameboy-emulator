import { cn } from "@callcc/toolkit-js/cn";
import type { SwitchProps } from "@radix-ui/react-switch";
import { Switch as RadixSwitch, SwitchThumb } from "@radix-ui/react-switch";

export function Switch(props: SwitchProps) {
  return (
    <RadixSwitch
      {...props}
      className={cn(
        props.className,
        "w-11 h-6 rounded-full relative block bg-primary/30",
        "data-[state=checked]:bg-primary",
      )}
    >
      <SwitchThumb
        className={cn(
          "block h-5 w-5 bg-white rounded-full translate-x-0.5 transition-transform duration-200",
          "data-[state=checked]:translate-x-5.5",
        )}
      />
    </RadixSwitch>
  );
}
