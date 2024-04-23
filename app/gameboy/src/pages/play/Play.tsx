import { AbButton } from "gameboy/components/AbButton";
import { DirectionButton } from "gameboy/components/DirectionButton";
import { FlexBox } from "gameboy/components/flex-box";
import { FnButton } from "gameboy/components/FnButton";
import { IconFullscreenExit } from "gameboy/components/icons";
import { Screen } from "gameboy/components/Screen";
import { GameBoyControl, JoypadKey } from "gameboy/gameboy";
import { useGamepadController } from "gameboy/hooks/useGamepadController";
import { useKeyboardController } from "gameboy/hooks/useKeyboardController";
import { ModalCanceledError } from "gameboy/model/error";
import { storage } from "gameboy/storage/indexdb";
import { store, actions, useAppStore } from "gameboy/store";
import { IGameBoyButton } from "gameboy/types";
import * as utils from "gameboy/utils";
import { cn } from "gameboy/utils/cn";
import type { CSSProperties } from "react";
import { useEffect, useRef } from "react";

import * as styles from "./Play.css";

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

export function Play(props: IPagePlayProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  const gameId = useAppStore((st) => {
    return st.games?.find((c) => c.id === st.selectedGameId)?.id;
  });

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!gameId || !canvas) {
      return;
    }

    let canceled = false;
    (async () => {
      const game = await storage.gameStore.queryById(gameId);
      const sav = game!.sav;
      const rom = await game!.rom
        .arrayBuffer()
        .then((buf) => new Uint8ClampedArray(buf));
      if (!canceled) {
        gameboy.uninstall();
        await gameboy.install(rom, canvas, 2, sav);
        const snapshot = store.getState().dialog.play.snapshot;
        if (snapshot && snapshot.gameId === gameId) {
          gameboy.restoreSnapshot(snapshot.data);
        }
        gameboy.play();
        await storage.gameStore.update({
          id: gameId,
          lastPlayTime: Date.now(),
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
      className={cn(styles.root, props.className)}
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
            style={{ marginTop: 140 }}
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
          gameboy.pause();
          actions
            .openExitConfirmModal()
            .then(async (action) => {
              const canvas = canvasRef.current;
              if (!gameId || !canvas) {
                return;
              }

              if (action === "snapshot" || action === "no_snapshot") {
                const sav = gameboy.createSav();
                if (sav) {
                  await storage.gameStore.update({ id: gameId, sav });
                }
              }

              if (action === "snapshot") {
                const snapshot = gameboy.takeSnapshot();
                const time = Date.now();
                const cover = await utils.canvasToBlob(
                  canvas,
                  "image/jpeg",
                  0.7,
                );
                storage.snapshotStore.insert({
                  data: snapshot,
                  gameId,
                  time,
                  name: "Snapshot",
                  cover,
                });
                actions.closePlayModal("snapshot");
              } else if (action === "no_snapshot") {
                actions.closePlayModal("no_snapshot");
              }
            })
            .catch((err) => {
              if (err instanceof ModalCanceledError) {
                gameboy.play();

                return;
              }

              throw err;
            });
        }}
      />
    </FlexBox>
  );
}
