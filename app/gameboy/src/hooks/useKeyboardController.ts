import type { GameBoyControl } from "gameboy/gameboy";
import { JoypadButton } from "gameboy/gameboy";
import { useEffect } from "react";
import { fromEvent, map, merge, filter, distinctUntilChanged } from "rxjs";

const keyMapping: Record<string, JoypadButton> = {
  ArrowRight: JoypadButton.Right,
  ArrowLeft: JoypadButton.Left,
  ArrowUp: JoypadButton.Up,
  ArrowDown: JoypadButton.Down,
  a: JoypadButton.A,
  s: JoypadButton.B,
  Enter: JoypadButton.Start,
  Shift: JoypadButton.Select,
};

export function useKeyboardController(props: {
  gameboy: GameBoyControl | undefined;
}) {
  const gameboy = props.gameboy;

  useEffect(() => {
    if (!gameboy) return;

    const isKeyWanted = (key: string) => Object.keys(keyMapping).includes(key);

    const keydown$ = fromEvent<KeyboardEvent>(document, "keydown").pipe(
      map((evt) => evt.key),
    );
    const keyup$ = fromEvent<KeyboardEvent>(document, "keyup").pipe(
      map((evt) => evt.key),
    );

    const keys$ = merge(
      keydown$.pipe(
        filter(isKeyWanted),
        map((key) => ({ key, pressed: true })),
      ),
      keyup$.pipe(
        filter(isKeyWanted),
        map((key) => ({ key, pressed: false })),
      ),
    );

    const keysSub = keys$
      .pipe(
        distinctUntilChanged(
          (prev, cur) => prev.key === cur.key && prev.pressed === cur.pressed,
        ),
      )
      .subscribe(({ key, pressed }) => {
        gameboy.changeButton(keyMapping[key], pressed);
      });

    return () => {
      keysSub.unsubscribe();
    };
  }, [gameboy]);
}
