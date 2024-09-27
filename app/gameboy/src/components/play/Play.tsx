import { AbButton } from "gameboy/components/core/AbButton";
import { DirectionButton } from "gameboy/components/core/DirectionButton";
import { FnButton } from "gameboy/components/core/FnButton";
import { Screen } from "gameboy/components/core/Screen";
import type { ISnapshotsModalRef } from "gameboy/components/snapshots-modal";
import { SnapshotsModal } from "gameboy/components/snapshots-modal";
import { useGamepadController } from "gameboy/hooks/useGamepadController";
import { useKeyboardController } from "gameboy/hooks/useKeyboardController";
import { storage } from "gameboy/storage/indexdb";
import { appStore, useAppStore } from "gameboy/store/app";
import { loadGames } from "gameboy/store/game";
import { settingsStore } from "gameboy/store/settings";
import type { CSSProperties } from "react";
import { forwardRef, useEffect, useRef } from "react";
import { PlayOperationBar } from "./PlayOperationBar";
import {
  deleteSnapshot,
  gameboy,
  handleButtonDown,
  handleButtonUp,
  switchSnapshot,
} from "./actions";
import type { IExitGameModalRef } from "./exit-game-modal";
import { ExitGameModal } from "./exit-game-modal";

export interface IPagePlayProps {
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
      if (!game) return;
      const sav = game.sav;
      const rom = await game.rom
        .arrayBuffer()
        .then((buf) => new Uint8ClampedArray(buf));
      if (!canceled) {
        gameboy.uninstall();
        await gameboy.install(rom, canvas, sav, dbgCanvas || undefined);
        const snapshot = appStore.getState().dialog.play.snapshot;
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
      loadGames();
    };
  }, [gameId]);

  useKeyboardController({ gameboy });
  useGamepadController({ gameboy });

  useEffect(() => {
    const handleVisibilityChange = () => {
      if (
        gameboy.state.status !== "uninstalled" &&
        settingsStore.getState().autoPause &&
        document.hidden
      ) {
        gameboy.pause();
      }
    };
    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      return document.removeEventListener(
        "visibilitychange",
        handleVisibilityChange,
      );
    };
  }, []);

  return (
    <>
      <div className="flex justify-end bg-bg" style={props.style}>
        <DebugCanvas ref={dbgCanvasRef} />
        <div className="flex justify-end basis-0 grow shrink-0">
          <div className="pt-5 pr-5">
            <DirectionButton onDown={handleButtonDown} onUp={handleButtonUp} />
          </div>
        </div>

        <div className="flex items-center shrink-0">
          <Screen ref={canvasRef} />
        </div>

        <div className="basis-0 grow shrink-0">
          <div className="pl-5 pt-5">
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
        </div>
      </div>

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
              icon: <i className="i-ic:baseline-delete-forever" />,
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
