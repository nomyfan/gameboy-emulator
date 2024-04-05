import { clsx } from "clsx";
import type { CSSProperties } from "react";

import type { IDirectionButton } from "../types";

import * as styles from "./DirectionButton.css";

function Button(props: {
  className?: string;
  style?: CSSProperties;
  onDown?: () => void;
  onUp?: () => void;
}) {
  return (
    <button
      className={clsx(styles.button, props.className)}
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
    <div className={styles.directionButton}>
      <Button
        key="top"
        style={{
          boxShadow: "-4px -4px 4px rgba(255,255,255,.25)",
          gridColumnStart: 2,
          borderBottomLeftRadius: 0,
          borderBottomRightRadius: 0,
        }}
        onDown={() => props.onDown?.("UP")}
        onUp={() => props.onUp?.("UP")}
      />
      <Button
        key="left"
        style={{
          boxShadow:
            "0px 4px 4px rgba(0,0,0,.25),-4px -4px 4px rgba(255,255,255,.25)",
          gridRowStart: 2,
          borderTopRightRadius: 0,
          borderBottomRightRadius: 0,
        }}
        onDown={() => props.onDown?.("LEFT")}
        onUp={() => props.onUp?.("LEFT")}
      />
      <div key="center" className={styles.center}>
        <div key="circle" className={styles.circle} />
      </div>
      <Button
        key="right"
        style={{
          boxShadow:
            "4px 0px 4px rgba(0,0,0,.25),0px 4px 4px rgba(0,0,0,.25),4px -4px 4px rgba(255,255,255,.25)",
          gridRowStart: 2,
          gridColumnStart: 3,
          borderTopLeftRadius: 0,
          borderBottomLeftRadius: 0,
        }}
        onDown={() => props.onDown?.("RIGHT")}
        onUp={() => props.onUp?.("RIGHT")}
      />
      <Button
        key="bottom"
        style={{
          boxShadow:
            "0px 4px 4px rgba(0,0,0,.25),-4px 4px 4px rgba(255,255,255,.25)",
          gridRowStart: 3,
          gridColumnStart: 2,
          borderTopLeftRadius: 0,
          borderTopRightRadius: 0,
        }}
        onDown={() => props.onDown?.("DOWN")}
        onUp={() => props.onUp?.("DOWN")}
      />
    </div>
  );
}

export { DirectionButton };
