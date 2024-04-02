import { JoypadKey } from "gb-wasm";
import { useRef } from "react";

import { AbButton } from "./components/AbButton";
import { DirectionButton } from "./components/DirectionButton";
import { FnButton } from "./components/FnButton";
import { Screen, SCALE } from "./components/Screen";
import { GameBoyControl } from "./gameboy";
import { useGamepadController } from "./hooks/useGamepadController";
import { useKeyboardController } from "./hooks/useKeyboardController";
import { cn } from "./lib/utils";
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
  const screenRef = useRef<HTMLCanvasElement>(null);

  useKeyboardController({ gameboy });
  useGamepadController({ gameboy });

  return (
    <div className={cn("min-h-screen bg-[#C8C4BE]")}>
      <Screen ref={screenRef} className="mb-[20px]" />
      <div className={cn("flex justify-between items-center px-5")}>
        <DirectionButton onDown={handleButtonDown} onUp={handleButtonUp} />
        <AbButton onDown={handleButtonDown} onUp={handleButtonUp} />
      </div>
      <div className="py-[30px]">
        <FnButton onUp={handleButtonUp} onDown={handleButtonDown} />
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
