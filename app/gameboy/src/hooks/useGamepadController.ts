import type { GameBoyControl } from "gameboy/gameboy";
import { JoypadButton } from "gb_wasm";
import { useEffect } from "react";
import { animationFrameScheduler } from "rxjs";

/**
 * @see https://w3c.github.io/gamepad/#remapping
 * @see https://support.xbox.com/en-US/help/hardware-network/controller/get-to-know-your-xbox-series-x-s-controller
 */
const xboxStandardMapping = [
  [12, JoypadButton.Up],
  [13, JoypadButton.Down],
  [14, JoypadButton.Left],
  [15, JoypadButton.Right],
  [0, JoypadButton.A],
  [1, JoypadButton.B],
  [8, JoypadButton.Select],
  [9, JoypadButton.Start],
];

/**
 * Only Xbox controller is supported now.
 * @param props
 */
function useGamepadController(props: { gameboy: GameBoyControl | undefined }) {
  const gameboy = props.gameboy;

  useEffect(() => {
    if (!gameboy) {
      return;
    }

    // Used to comparing with the latest state for reducing API calling.
    let prevState = 0;
    const sub = animationFrameScheduler.schedule(function () {
      const gamepad = navigator
        .getGamepads()
        ?.find(
          (gamepad) =>
            gamepad &&
            gamepad.mapping === "standard" &&
            gamepad.id.toLocaleLowerCase().includes("xbox"),
        );
      if (!gamepad) {
        this.schedule();
        return;
      }

      const newState = xboxStandardMapping.reduce((state, [index, key]) => {
        if (gamepad.buttons[index].pressed) {
          state |= key;
        }
        return state;
      }, 0);

      if (prevState !== newState) {
        gameboy.changeButtons(newState);
        prevState = newState;
      }

      this.schedule();
    });

    return () => sub.unsubscribe();
  }, [gameboy]);
}

export { useGamepadController };
