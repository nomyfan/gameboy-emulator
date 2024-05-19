import { AbButton } from "gameboy/components/core/AbButton";
import { DirectionButton } from "gameboy/components/core/DirectionButton";
import { FlexBox } from "gameboy/components/core/flex-box";
import { FnButton } from "gameboy/components/core/FnButton";
import { Screen } from "gameboy/components/core/Screen";
import { IconDelete } from "gameboy/components/icons";
import type { ISnapshotsModalRef } from "gameboy/components/snapshots-modal";
import { SnapshotsModal } from "gameboy/components/snapshots-modal";
import { useGamepadController } from "gameboy/hooks/useGamepadController";
import { useKeyboardController } from "gameboy/hooks/useKeyboardController";
import { storage } from "gameboy/storage/indexdb";
import { store, actions, useAppStore } from "gameboy/store";
import { cn } from "gameboy/utils/cn";
import type { CSSProperties } from "react";
import { useEffect, useRef, forwardRef } from "react";

import {
  handleButtonDown,
  handleButtonUp,
  gameboy,
  switchSnapshot,
  deleteSnapshot,
} from "./actions";
import type { IExitGameModalRef } from "./exit-game-modal";
import { ExitGameModal } from "./exit-game-modal";
import * as styles from "./Play.css";
import { PlayOperationBar } from "./PlayOperationBar";

export interface IPagePlayProps {
  className?: string;
  style?: CSSProperties;
}

const DebugCanvas = forwardRef<HTMLCanvasElement, unknown>(
  function DebugCanvas(_, ref) {
    const showDebugCanvas =
      new URLSearchParams(window.location.search).get("dbg") === "+";

    return showDebugCanvas ? (
      <canvas
        ref={ref}
        style={{ position: "absolute", left: 10, bottom: 10 }}
        height={256 + 10 + 256}
        width={256 + 10 + 40}
      />
    ) : null;
  },
);

export function Play(props: IPagePlayProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const dbgCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const exitGameModalRef = useRef<IExitGameModalRef>(null);
  const snapshotsModalRef = useRef<ISnapshotsModalRef>(null);

  const gameId = useAppStore((st) => st.selectedGameId);

  useEffect(() => {
    const canvas = canvasRef.current;
    const dbgCanvas = dbgCanvasRef.current;
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
        await gameboy.install(rom, canvas, sav, dbgCanvas || undefined);
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
    <>
      <FlexBox
        justify="right"
        className={cn(styles.root, props.className)}
        style={props.style}
      >
        <DebugCanvas ref={dbgCanvasRef} />
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
      </FlexBox>

      <PlayOperationBar
        canvasRef={canvasRef}
        snapshotsModalRef={snapshotsModalRef}
        exitGameModalRef={exitGameModalRef}
      />

      <ExitGameModal ref={exitGameModalRef} />
      <SnapshotsModal
        ref={snapshotsModalRef}
        snapshotsProps={{
          actionItems: [
            {
              label: "加载存档",
              onClick: async (snapshot) => {
                await switchSnapshot(snapshot.data);
              },
            },
            {
              icon: <IconDelete />,
              label: "删除",
              alert: true,
              onClick: async (snapshot, { refresh }) => {
                await deleteSnapshot(snapshot.id);
                await refresh();
              },
            },
          ],
        }}
      />
    </>
  );
}
