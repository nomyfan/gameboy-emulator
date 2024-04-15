import clsx from "clsx";
import { JoypadKey } from "gb-wasm";
import { useRef } from "react";
import { useOrientation } from "react-use";

import * as styles from "./App-legacy.css";
import { AbButton } from "./components/AbButton";
import { DirectionButton } from "./components/DirectionButton";
import { FnButton } from "./components/FnButton";
import { Screen, SCALE } from "./components/Screen";
import { GameBoyControl } from "./gameboy";
import { useGamepadController } from "./hooks/useGamepadController";
import { useKeyboardController } from "./hooks/useKeyboardController";
import { IGameBoyButton } from "./types";

const gameboy = new GameBoyControl();

const handleButtonChange = (button: IGameBoyButton, pressed: boolean) => {
  let key: JoypadKey;
  switch (button) {
    case "B":
      key = JoypadKey.B;
      break;
    case "A":
      key = JoypadKey.A;
      break;
    case "LEFT":
      key = JoypadKey.Left;
      break;
    case "RIGHT":
      key = JoypadKey.Right;
      break;
    case "UP":
      key = JoypadKey.Up;
      break;
    case "DOWN":
      key = JoypadKey.Down;
      break;
    case "SELECT":
      key = JoypadKey.Select;
      break;
    case "START":
      key = JoypadKey.Start;
      break;
    default: {
      const wrongButton: never = button;
      throw new Error("Wrong button value " + wrongButton);
    }
  }
  gameboy.changeKey(key, pressed);
};

function handleButtonDown(button: IGameBoyButton) {
  handleButtonChange(button, true);
}

function handleButtonUp(button: IGameBoyButton) {
  handleButtonChange(button, false);
}

function App() {
  const appRef = useRef<HTMLDivElement>(null);
  const screenRef = useRef<HTMLCanvasElement>(null);

  useKeyboardController({ gameboy });
  useGamepadController({ gameboy });

  const orientation = useOrientation();
  const isLandscape =
    orientation.type === "landscape-primary" ||
    orientation.type === "landscape-secondary";

  const renderLandscape = () => {
    return (
      <>
        <Screen
          ref={screenRef}
          style={{ marginBottom: 20 }}
          left={
            <div
              style={{
                flexBasis: 0,
                flexShrink: 0,
                flexGrow: 1,
                display: "flex",
                justifyContent: "left",
              }}
            >
              <DirectionButton
                onDown={handleButtonDown}
                onUp={handleButtonUp}
              />
            </div>
          }
          right={
            <div
              style={{
                position: "relative",
                flexBasis: 0,
                flexGrow: 1,
                flexShrink: 0,
                display: "flex",
                justifyContent: "right",
                padding: 15,
              }}
            >
              <AbButton
                style={{
                  transform: "rotate(-25deg) translateY(-12px)",
                }}
                onDown={handleButtonDown}
                onUp={handleButtonUp}
              />
              <FnButton
                style={{
                  position: "absolute",
                  bottom: -80,
                }}
                onUp={handleButtonUp}
                onDown={handleButtonDown}
              />
            </div>
          }
        />
      </>
    );
  };

  const renderPortrait = () => {
    return (
      <>
        <Screen ref={screenRef} style={{ marginBottom: 20 }} />
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            paddingLeft: 20,
            paddingRight: 20,
          }}
        >
          <DirectionButton onDown={handleButtonDown} onUp={handleButtonUp} />
          <AbButton
            style={{
              transform: "rotate(-25deg) translateY(-12px)",
            }}
            onDown={handleButtonDown}
            onUp={handleButtonUp}
          />
        </div>
        <div
          style={{
            paddingTop: 30,
            paddingBottom: 30,
          }}
        >
          <FnButton onUp={handleButtonUp} onDown={handleButtonDown} />
        </div>
      </>
    );
  };

  const render = () => {
    if (isLandscape) {
      return renderLandscape();
    } else {
      return renderPortrait();
    }
  };

  return (
    <div ref={appRef} className={clsx(styles.app)}>
      {render()}
      <input
        type="file"
        accept=".gb"
        onChange={async (evt) => {
          const file = evt.target.files?.[0];
          if (!file || !screenRef.current) {
            return;
          }

          const buffer = new Uint8ClampedArray(await file.arrayBuffer());
          const canvas = screenRef.current;
          gameboy.uninstall();
          gameboy.install(buffer, canvas, SCALE);
          gameboy.play();
        }}
      />
      <button
        onClick={() => {
          const bytes = gameboy.takeSnapshot();
          const blob = new Blob([bytes], { type: "application/octet-stream" });
          const url = URL.createObjectURL(blob);
          const a = document.createElement("a");
          a.href = url;
          a.style.display = "none";
          a.download = "snapshot.ss";
          a.click();
          setTimeout(() => {
            a.remove();
            URL.revokeObjectURL(url);
          }, 1000);
        }}
      >
        Take snapshot
      </button>
      <input
        type="file"
        accept=".ss"
        onChange={(evt) => {
          const file = evt.target.files?.[0];
          if (!file) {
            return;
          }

          const reader = new FileReader();
          reader.onload = () => {
            const buffer = new Uint8Array(reader.result as ArrayBuffer);
            try {
              gameboy.restoreSnapshot(buffer);
              evt.target.value = "";
            } catch (err) {
              if (err instanceof Error) {
                const message = err.message;
                const parseGameBoyError = (errorMessage: string) => {
                  const RE = /^\[(E\w+?\d+?)\]/;
                  const match = RE.exec(errorMessage);
                  if (match) {
                    const code = match[1];
                    const message = errorMessage.replace(RE, "");
                    return { code, message };
                  } else {
                    return null;
                  }
                };
                const gbError = parseGameBoyError(message);
                if (gbError) {
                  console.error(gbError);
                  return;
                }
              }

              throw err;
            }
          };
          reader.readAsArrayBuffer(file);
        }}
      />
      <button
        onClick={() => {
          appRef.current?.requestFullscreen();
        }}
      >
        Fullscreen
      </button>
    </div>
  );
}

export { App };
