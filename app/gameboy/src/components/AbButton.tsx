import { clsx } from "clsx";
import type { CSSProperties, ReactNode } from "react";

import type { IAbButton } from "../types";

import * as styles from "./AbButton.css";

function Button(props: {
  style?: CSSProperties;
  onDown?: () => void;
  onUp?: () => void;
}) {
  return (
    <button
      className={styles.button}
      style={props.style}
      onMouseDown={() => props.onDown?.()}
      onTouchStart={() => props.onDown?.()}
      onMouseUp={() => props.onUp?.()}
      onTouchEnd={() => props.onUp?.()}
    />
  );
}

function ButtonLabel(props: { className?: string; children?: ReactNode }) {
  return (
    <label className={clsx(styles.buttonLabel, props.className)}>
      {props.children}
    </label>
  );
}

function AbButton(props: {
  onDown?: (button: IAbButton) => void;
  onUp?: (button: IAbButton) => void;
}) {
  return (
    <div className={styles.abButton}>
      <div className={styles.buttonGroup}>
        <Button
          style={{ marginRight: "20px" }}
          onDown={() => props.onDown?.("B")}
          onUp={() => props.onUp?.("B")}
        />
        <Button
          onDown={() => props.onDown?.("A")}
          onUp={() => props.onUp?.("A")}
        />
      </div>

      <div className={styles.labelGroup}>
        <ButtonLabel>B</ButtonLabel>
        <ButtonLabel>A</ButtonLabel>
      </div>
    </div>
  );
}

export { AbButton };
