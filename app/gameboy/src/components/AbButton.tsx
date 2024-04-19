import { clsx } from "clsx";
import type { IAbButton } from "gameboy/types";
import type { CSSProperties, ReactNode } from "react";

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

function ButtonLabel(props: {
  className?: string;
  children?: ReactNode;
  style?: CSSProperties;
}) {
  return (
    <label
      className={clsx(styles.buttonLabel, props.className)}
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
    <div className={styles.abButton} style={props.style}>
      <div className={styles.buttonGroup}>
        <Button
          onDown={() => props.onDown?.("B")}
          onUp={() => props.onUp?.("B")}
        />
        <div style={{ width: GAP }} />
        <Button
          onDown={() => props.onDown?.("A")}
          onUp={() => props.onUp?.("A")}
        />

        <div className={styles.labelGroup}>
          <ButtonLabel style={{ width: LABEL_WIDTH }}>B</ButtonLabel>
          <ButtonLabel style={{ width: LABEL_WIDTH }}>A</ButtonLabel>
        </div>
      </div>
    </div>
  );
}

export { AbButton };
