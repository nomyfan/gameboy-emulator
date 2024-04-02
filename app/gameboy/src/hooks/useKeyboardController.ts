import { useEffect } from "react";
import { fromEvent, map, merge, filter, distinctUntilChanged } from "rxjs";

import type { GameBoyControl } from "../gameboy";
import { JoypadKey } from "../gameboy";

const keyMapping: Record<string, JoypadKey> = {
  ArrowRight: JoypadKey.Right,
  ArrowLeft: JoypadKey.Left,
  ArrowUp: JoypadKey.Up,
  ArrowDown: JoypadKey.Down,
  a: JoypadKey.A,
  s: JoypadKey.B,
  Enter: JoypadKey.Start,
  Shift: JoypadKey.Select,
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
        gameboy.changeKey(keyMapping[key], pressed);
      });

    return () => {
      keysSub.unsubscribe();
    };
  }, [gameboy]);
}
