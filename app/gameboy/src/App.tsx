import { JoypadKey } from "gb-wasm";
import { useCallback, useRef, useState } from "react";

import { AbButton } from "./components/AbButton";
import { DirectionButton } from "./components/DirectionButton";
import { FnButton } from "./components/FnButton";
import { Screen } from "./components/Screen";
import { GameBoySupervisor } from "./gameboy-workers-supervisor";
import { useGamepadController } from "./hooks/useGamepadController";
import { useKeyboardController } from "./hooks/useKeyboardController";
import { cn } from "./lib/utils";

function App() {
  const ref = useRef<{
    newCanvasHandle: () => HTMLCanvasElement;
  }>(null);
  const [scale] = useState(2);
  const [supervisor, setSupervisor] = useState<GameBoySupervisor>();

  useKeyboardController({ supervisor: supervisor });
  useGamepadController({ supervisor: supervisor });

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
      if (pressed) {
        supervisor?.pressKey(key);
      } else {
        supervisor?.releaseKey(key);
      }
    },
    [supervisor],
  );

  return (
    <div className={cn("min-h-screen bg-[#C8C4BE]")}>
      <Screen ref={ref} className="mb-[20px]" />
      <div className={cn("flex justify-between items-center px-5")}>
        <DirectionButton
          onDown={(button) => {
            handleButtonChange(button, true);
          }}
          onUP={(button) => {
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
      <button
        onClick={() => {
          supervisor?.play();
        }}
      >
        Play
      </button>
      <button
        onClick={() => {
          supervisor?.pause();
        }}
      >
        Pause
      </button>
      <input
        type="file"
        accept=".gb"
        onChange={async (evt) => {
          const file = evt.target.files?.[0];
          if (!file || !ref.current) {
            return;
          }

          const canvas = ref.current.newCanvasHandle();
          const offscreen = canvas.transferControlToOffscreen();

          if (supervisor) {
            await supervisor.terminate();
            await supervisor.install(file, offscreen, scale);
          } else {
            const createdSupervisor = await GameBoySupervisor.create();
            await createdSupervisor.install(file, offscreen, scale);
            setSupervisor(createdSupervisor);
          }
        }}
      />
    </div>
  );
}

export { App };
