import { clsx } from "clsx";
import { lightShadow, darkShadow } from "gameboy/styles";
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
          boxShadow: lightShadow("-4px -4px 4px"),
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
          boxShadow: `${darkShadow("0px 4px 4px")}, ${lightShadow("-4px -4px 4px")}`,
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
          boxShadow: `${darkShadow("4px 0px 4px")}, ${darkShadow("0px 4px 4px")}, ${lightShadow("4px -4px 4px")}`,
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
          boxShadow: `${darkShadow("0 4px 4px")}, ${lightShadow("-4px 4px 4px")}`,
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
