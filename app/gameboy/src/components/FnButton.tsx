import type { IFnButton } from "gameboy/types";
import type { CSSProperties } from "react";

import * as styles from "./FnButton.css";

function Button(props: {
  label: string;
  className?: string;
  style?: CSSProperties;
  onDown?: () => void;
  onUp?: () => void;
}) {
  return (
    <div className={styles.button} style={props.style}>
      <button
        onMouseDown={() => props.onDown?.()}
        onTouchStart={() => props.onDown?.()}
        onMouseUp={() => props.onUp?.()}
        onTouchEnd={() => props.onUp?.()}
      />
      <label>{props.label}</label>
    </div>
  );
}

function FnButton(props: {
  onDown?: (button: IFnButton) => void;
  onUp?: (button: IFnButton) => void;
  style?: CSSProperties;
}) {
  return (
    <div className={styles.fnButton} style={props.style}>
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
