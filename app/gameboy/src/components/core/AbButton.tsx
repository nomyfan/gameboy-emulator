import type { IAbButton } from "gameboy/types";
import type { CSSProperties, ReactNode } from "react";

function Button(props: {
  style?: CSSProperties;
  onDown?: () => void;
  onUp?: () => void;
}) {
  return (
    <button
      className="rounded-full bg-[#9B0757] w-[50px] h-[50px] shadow-[3px_3px_4px_rgba(0,0,0,0.25)]"
      style={props.style}
      onMouseDown={() => props.onDown?.()}
      onTouchStart={() => props.onDown?.()}
      onMouseUp={() => props.onUp?.()}
      onTouchEnd={() => props.onUp?.()}
    />
  );
}

function ButtonLabel(props: { children?: ReactNode; style?: CSSProperties }) {
  return (
    <label
      className="w-[45px] text-shadow-[-2px_-2px_4px_rgba(255,255,255,0.25),3px_3px_4px_rgba(0,0,0,0.25)]"
      style={props.style}
    >
      {props.children}
    </label>
  );
}

function AbButton(props: {
  onDown?: (button: IAbButton) => void;
  onUp?: (button: IAbButton) => void;
  style?: CSSProperties;
}) {
  const GAP = 20;
  const LABEL_WIDTH = `calc(50% - ${GAP / 2}px)`;

  return (
    <div className="font-bold" style={props.style}>
      <div className="relative flex w-fit">
        <Button
          onDown={() => props.onDown?.("B")}
          onUp={() => props.onUp?.("B")}
        />
        <div style={{ width: GAP }} />
        <Button
          onDown={() => props.onDown?.("A")}
          onUp={() => props.onUp?.("A")}
        />

        <div className="flex justify-between text-center w-full absolute -bottom-[30px]">
          <ButtonLabel style={{ width: LABEL_WIDTH }}>B</ButtonLabel>
          <ButtonLabel style={{ width: LABEL_WIDTH }}>A</ButtonLabel>
        </div>
      </div>
    </div>
  );
}

export { AbButton };
