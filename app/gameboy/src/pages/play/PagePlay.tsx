import { clsx } from "clsx";
import { CSSProperties, useEffect, useRef } from "react";
import { useStore } from "zustand";

import { AbButton } from "../../components/AbButton";
import { DirectionButton } from "../../components/DirectionButton";
import { FlexBox } from "../../components/flex-box";
import { FnButton } from "../../components/FnButton";
import { IconFullscreenExit } from "../../components/icons";
import { Screen } from "../../components/Screen";
import { GameBoyControl, JoypadKey } from "../../gameboy";
import { useGamepadController } from "../../hooks/useGamepadController";
import { useKeyboardController } from "../../hooks/useKeyboardController";
import { storage } from "../../storage/indexdb";
import { store, actions } from "../../store";
import { rem } from "../../styles";
import { IGameBoyButton } from "../../types";

import * as styles from "./PagePlay.css";

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

export interface IPagePlayProps {
  className?: string;
  style?: CSSProperties;
}

export function PagePlay(props: IPagePlayProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  const gameId = useStore(store, (st) => {
    return st.games?.find((c) => c.id === st.selectedGameId)?.id;
  });

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!gameId || !canvas) {
      return;
    }

    let canceled = false;
    (async () => {
      const file = await storage.gameStore.queryById(gameId);
      if (!file) {
        // TODO:
        return;
      }
      const rom = await file.rom
        .arrayBuffer()
        .then((buf) => new Uint8ClampedArray(buf));
      if (!canceled) {
        gameboy.uninstall();
        gameboy.install(rom, canvas, 2);
        gameboy.play();
        await storage.gameStore.update({
          id: gameId,
          last_play_time: Date.now(),
        });
      }
    })();

    return () => {
      canceled = true;
      gameboy.uninstall();
      actions.loadGames();
    };
  }, [gameId]);

  useKeyboardController({ gameboy });
  useGamepadController({ gameboy });

  return (
    <FlexBox
      justify="right"
      className={clsx(styles.root, props.className)}
      style={props.style}
    >
      <FlexBox justify="end" className={styles.side}>
        <div className={styles.leftSide}>
          <DirectionButton onDown={handleButtonDown} onUp={handleButtonUp} />
        </div>
      </FlexBox>

      <FlexBox align="center" className={styles.screen}>
        <Screen ref={canvasRef} />
      </FlexBox>

      <FlexBox className={styles.side}>
        <div className={styles.rightSide}>
          <AbButton
            style={{ transform: "rotate(-25deg)" }}
            onDown={handleButtonDown}
            onUp={handleButtonUp}
          />
          <FnButton
            style={{ marginTop: rem(400) }}
            onDown={handleButtonDown}
            onUp={handleButtonUp}
          />
        </div>
      </FlexBox>

      <IconFullscreenExit
        style={{
          position: "absolute",
          right: 10,
          top: 10,
          height: 36,
          width: 36,
        }}
        onClick={() => {
          document.exitFullscreen();
          actions.togglePlayModal(false);
        }}
      />
    </FlexBox>
  );
}
