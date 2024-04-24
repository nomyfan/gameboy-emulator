import type { IDirectionButton } from "gameboy/types";
import type { CSSProperties } from "react";

import * as styles from "./DirectionButton.css";

function Button(props: {
  className?: string;
  style?: CSSProperties;
  onDown?: () => void;
  onUp?: () => void;
}) {
  return (
    <button
      className={props.className}
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
        className={styles.buttonTop}
        onDown={() => props.onDown?.("UP")}
        onUp={() => props.onUp?.("UP")}
      />
      <Button
        key="left"
        className={styles.buttonLeft}
        onDown={() => props.onDown?.("LEFT")}
        onUp={() => props.onUp?.("LEFT")}
      />
      <div key="center" className={styles.center}>
        <div key="circle" className={styles.circle} />
      </div>
      <Button
        key="right"
        className={styles.buttonRight}
        onDown={() => props.onDown?.("RIGHT")}
        onUp={() => props.onUp?.("RIGHT")}
      />
      <Button
        key="bottom"
        className={styles.buttonBottom}
        onDown={() => props.onDown?.("DOWN")}
        onUp={() => props.onUp?.("DOWN")}
      />
    </div>
  );
}

export { DirectionButton };
