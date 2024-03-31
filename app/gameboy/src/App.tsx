import { useRef, useState } from "react";

import { GameBoySupervisor } from "./gameboy-workers-supervisor";
import { useGamepadController } from "./hooks/useGamepadController";
import { useKeyboardController } from "./hooks/useKeyboardController";

const RESOLUTION_X = 160;
const RESOLUTION_Y = 144;

function App() {
  const ref = useRef<HTMLCanvasElement>(null);
  const [scale] = useState(2);
  const [supervisor, setSupervisor] = useState<GameBoySupervisor>();

  useKeyboardController({ supervisor: supervisor });
  useGamepadController({ supervisor: supervisor });

  return (
    <div>
      <canvas
        ref={ref}
        width={RESOLUTION_X * scale}
        height={RESOLUTION_Y * scale}
      />
      <br />
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
