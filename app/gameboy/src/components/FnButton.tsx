import { cn } from "../lib/utils";
import type { IFnButton } from "../types";

function Button(props: {
  label: string;
  className?: string;
  onDown?: () => void;
  onUp?: () => void;
}) {
  return (
    <div
      className={cn("w-fit", props.className)}
      style={{
        transform: "rotate(-25deg)",
      }}
    >
      <button
        className={cn("h-[15px] w-[65px] bg-[#9F9AAF] block rounded-[4px]")}
        style={{
          boxShadow:
            "-2px -2px 4px rgba(255,255,255,.25), 2px 2px 4px rgba(0,0,0,.25)",
        }}
        onMouseDown={() => props.onDown?.()}
        onTouchStart={() => props.onDown?.()}
        onMouseUp={() => props.onUp?.()}
        onTouchEnd={() => props.onUp?.()}
      />
      <label
        className={cn("font-semibold text-[12px] block w-full text-center")}
        style={{
          textShadow:
            "-2px -2px 4px rgba(255,255,255,.25), 2px 2px 4px rgba(0,0,0,.25)",
        }}
      >
        {props.label}
      </label>
    </div>
  );
}

function FnButton(props: {
  onDown?: (button: IFnButton) => void;
  onUp?: (button: IFnButton) => void;
}) {
  return (
    <div className={cn("flex justify-center")}>
      <Button
        label="SELECT"
        className="mr-[20px]"
        onDown={() => props.onDown?.("SELECT")}
        onUp={() => props.onUp?.("SELECT")}
      />
      <Button
        label="START"
        onDown={() => props.onDown?.("START")}
        onUp={() => props.onUp?.("START")}
      />
    </div>
  );
}
export { FnButton };
