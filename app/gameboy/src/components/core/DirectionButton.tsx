import type { IDirectionButton } from "gameboy/types";
import { cn } from "gameboy/utils/cn";
import type { CSSProperties } from "react";

function Button(props: {
  className?: string;
  style?: CSSProperties;
  onDown?: () => void;
  onUp?: () => void;
}) {
  return (
    <button
      className={cn("bg-black w-full h-full rounded-1", props.className)}
      style={props.style}
      onMouseDown={() => props.onDown?.()}
      onTouchStart={() => props.onDown?.()}
      onMouseUp={() => props.onUp?.()}
      onTouchEnd={() => props.onUp?.()}
    />
  );
}

function DirectionButton(props: {
  onDown?: (button: IDirectionButton) => void;
  onUp?: (button: IDirectionButton) => void;
}) {
  return (
    <div className="grid rows-[44px_40px_44px] cols-[44px_40px_44px]">
      <Button
        key="top"
        className="col-start-2 shadow-[-4px_-4px_4px] shadow-white/25"
        style={{
          borderBottomLeftRadius: 0,
          borderBottomRightRadius: 0,
        }}
        onDown={() => props.onDown?.("Up")}
        onUp={() => props.onUp?.("Up")}
      />
      <Button
        key="left"
        className="row-start-2 shadow-[0_4px_4px_rgba(0,0,0,0.25),-4px_-4px_4px_rgba(255,255,255,0.25)]"
        style={{
          borderTopRightRadius: 0,
          borderBottomRightRadius: 0,
        }}
        onDown={() => props.onDown?.("Left")}
        onUp={() => props.onUp?.("Left")}
      />
      <div
        key="center"
        className="flex-center bg-black row-start-2 col-start-2"
      >
        <div
          key="circle"
          className="h-30px w-30px rounded-full bg-#e3e1dd shadow-[inset_-4px_-4px_4px_rgba(255,255,255,.25),inset_4px_4px_4px_rgba(0,0,0,.25)]"
        />
      </div>
      <Button
        key="right"
        className="row-start-2 col-start-3 shadow-[4px_0_4px_rgba(0,0,0,0.25),0_4px_4px_rgba(0,0,0,0.25),4px_-4px_4px_rgba(255,255,255,0.25)]"
        style={{
          borderTopLeftRadius: 0,
          borderBottomLeftRadius: 0,
        }}
        onDown={() => props.onDown?.("Right")}
        onUp={() => props.onUp?.("Right")}
      />
      <Button
        key="bottom"
        className="row-start-3 col-start-2 shadow-[0_4px_4px_rgba(0,0,0,0.25),-4px_4px_4px_rgba(255,255,255,0.25)]"
        style={{
          borderTopLeftRadius: 0,
          borderTopRightRadius: 0,
        }}
        onDown={() => props.onDown?.("Down")}
        onUp={() => props.onUp?.("Down")}
      />
    </div>
  );
}

export { DirectionButton };
