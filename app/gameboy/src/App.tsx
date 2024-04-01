import { useRef, useState } from "react";

import { AbButton } from "./components/AbButton";
import { DirectionButton } from "./components/DirectionButton";
import { Screen } from "./components/Screen";
import { GameBoySupervisor } from "./gameboy-workers-supervisor";
import { useGamepadController } from "./hooks/useGamepadController";
import { useKeyboardController } from "./hooks/useKeyboardController";
import { cn } from "./lib/utils";

function App() {
  const ref = useRef<HTMLCanvasElement>(null);
  const [scale] = useState(2);
  const [supervisor, setSupervisor] = useState<GameBoySupervisor>();

  useKeyboardController({ supervisor: supervisor });
  useGamepadController({ supervisor: supervisor });

  return (
    <div className={cn("min-h-screen bg-[#C8C4BE]")}>
      <Screen ref={ref} className="mb-[20px]" />
      <div className={cn("flex justify-between items-center px-5")}>
        <DirectionButton />
        <AbButton />
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
          if (!file) return;

          const canvas = ref.current!;
          // FIXME: can only transfer once
          const offscreen = canvas.transferControlToOffscreen();

          if (supervisor) {
            supervisor.install(file, offscreen, scale);
          } else {
            const createdSupervisor = await GameBoySupervisor.create();
            createdSupervisor.install(file, offscreen, scale);
            setSupervisor(createdSupervisor);
          }
        }}
      />
    </div>
  );
}

export { App };
