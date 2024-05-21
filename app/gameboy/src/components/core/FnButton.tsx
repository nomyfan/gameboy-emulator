import type { IFnButton } from "gameboy/types";
import type { CSSProperties } from "react";

function Button(props: {
  label: string;
  style?: CSSProperties;
  onDown?: () => void;
  onUp?: () => void;
}) {
  return (
    <div className="w-fit transform-rotate--25deg" style={props.style}>
      <button
        className="h-15px w-65px bg-#9f9aaf block rounded-1 shadow-[-2px_-2px_4px_rgba(255,255,255,0.25),2px_2px_4px_rgba(0,0,0,0.25)]"
        onMouseDown={() => props.onDown?.()}
        onTouchStart={() => props.onDown?.()}
        onMouseUp={() => props.onUp?.()}
        onTouchEnd={() => props.onUp?.()}
      />
      <label className="font-bold text-12px block w-full text-center text-shadow-[-2px_-2px_4px_rgba(255,255,255,0.25),2px_2px_4px_rgba(0,0,0,0.25)]">
        {props.label}
      </label>
    </div>
  );
}

function FnButton(props: {
  onDown?: (button: IFnButton) => void;
  onUp?: (button: IFnButton) => void;
  style?: CSSProperties;
}) {
  return (
    <div className="flex justify-center" style={props.style}>
      <Button
        label="SELECT"
        style={{ marginRight: 20 }}
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
