import { JoypadKey } from "gb-wasm";
import { useCallback, useRef } from "react";

import { AbButton } from "./components/AbButton";
import { DirectionButton } from "./components/DirectionButton";
import { FnButton } from "./components/FnButton";
import { Screen, SCALE } from "./components/Screen";
import { GameBoyControl } from "./gameboy";
import { useGamepadController } from "./hooks/useGamepadController";
import { useKeyboardController } from "./hooks/useKeyboardController";
import { cn } from "./lib/utils";

const gameboy = new GameBoyControl();

function App() {
  const screenRef = useRef<HTMLCanvasElement>(null);

  useKeyboardController({ gameboy });
  useGamepadController({ gameboy });

  const handleButtonChange = useCallback(
    (
      button: "UP" | "RIGHT" | "DOWN" | "LEFT" | "A" | "B" | "START" | "SELECT",
      pressed: boolean,
    ) => {
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
    },
    [],
  );

  return (
    <div className={cn("min-h-screen bg-[#C8C4BE]")}>
      <Screen ref={screenRef} className="mb-[20px]" />
      <div className={cn("flex justify-between items-center px-5")}>
        <DirectionButton
          onDown={(button) => {
            handleButtonChange(button, true);
          }}
          onUp={(button) => {
            handleButtonChange(button, false);
          }}
        />
        <AbButton
          onDown={(button) => {
            handleButtonChange(button, true);
          }}
          onUp={(button) => {
            handleButtonChange(button, false);
          }}
        />
      </div>
      <div className="py-[30px]">
        <FnButton
          onUp={(button) => {
            handleButtonChange(button, true);
          }}
          onDown={(button) => {
            handleButtonChange(button, false);
          }}
        />
      </div>
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
    </div>
  );
}

export { App };
