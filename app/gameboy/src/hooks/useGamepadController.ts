import type { GameBoyControl } from "gameboy/gameboy";
import { JoypadKey } from "gb_wasm";
import { useEffect } from "react";
import { animationFrameScheduler } from "rxjs";

/**
 * @see https://w3c.github.io/gamepad/#remapping
 * @see https://support.xbox.com/en-US/help/hardware-network/controller/get-to-know-your-xbox-series-x-s-controller
 */
const xboxStandardMapping = [
  [12, JoypadKey.Up],
  [13, JoypadKey.Down],
  [14, JoypadKey.Left],
  [15, JoypadKey.Right],
  [0, JoypadKey.A],
  [1, JoypadKey.B],
  [8, JoypadKey.Select],
  [9, JoypadKey.Start],
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
        gameboy.changeKeyState(newState);
        prevState = newState;
      }

      this.schedule();
    });

    return () => sub.unsubscribe();
  }, [gameboy]);
}

export { useGamepadController };
