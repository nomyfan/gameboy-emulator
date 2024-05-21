import { OperationBar } from "gameboy/components/core/operation-bar";
import {
  IconExitToApp,
  IconExpandDown,
  IconHistory,
  IconPause,
  IconPlay,
  IconSave,
  IconSettings,
  IconVolumeOff,
  IconVolumeOn,
} from "gameboy/components/icons";
import type { IExitGameModalRef } from "gameboy/components/play/exit-game-modal";
import type { ISnapshotsModalRef } from "gameboy/components/snapshots-modal";
import { useToast } from "gameboy/components/toast/useToast";
import { ModalCanceledError } from "gameboy/model/error";
import { storage } from "gameboy/storage/indexdb";
import { useAppStore, actions } from "gameboy/store";
import type { RefObject } from "react";
import { useMemo, useState } from "react";
import { useStore } from "zustand";

import { gameboy, takeSnapshot } from "./actions";

export function PlayOperationBar(props: {
  canvasRef: RefObject<HTMLCanvasElement>;
  snapshotsModalRef: RefObject<ISnapshotsModalRef>;
  exitGameModalRef: RefObject<IExitGameModalRef>;
}) {
  const canvasRef = props.canvasRef;
  const snapshotsModalRef = props.snapshotsModalRef;
  const exitGameModalRef = props.exitGameModalRef;

  const { addToast } = useToast();
  const playing = useStore(gameboy.store, (st) => st.status === "playing");
  const muted = useStore(gameboy.store, (st) => st.muted);
  const gameId = useAppStore((st) => st.selectedGameId);

  const items = useMemo(() => {
    return [
      [
        {
          id: "mute-unmute",
          icon: muted ? <IconVolumeOn /> : <IconVolumeOff />,
          onClick: () => {
            gameboy.mute();
          },
        },
        {
          id: "take-snapshot",
          icon: <IconSave />,
          onClick: async () => {
            await takeSnapshot(canvasRef.current, gameId);
            addToast("已创建存档");
          },
        },
        {
          id: "snapshots",
          icon: <IconHistory />,
          onClick: () => {
            snapshotsModalRef.current?.open();
          },
        },
        {
          id: "play-pause",
          icon: playing ? <IconPause /> : <IconPlay />,
          onClick: () => {
            if (playing) {
              gameboy.pause();
            } else {
              gameboy.play();
            }
          },
        },
        {
          id: "exit",
          icon: <IconExitToApp style={{ transform: "rotate(180deg)" }} />,
          alert: true,
          onClick: async () => {
            gameboy.pause();
            exitGameModalRef
              .current!.open()
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
                  await takeSnapshot(canvasRef.current, gameId);
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
          },
        },
        {
          id: "settings",
          icon: <IconSettings />,
          onClick: async () => {
            gameboy.pause();
            await actions.openSettingsModal();
            gameboy.play();
          },
        },
      ],
    ];
  }, [
    addToast,
    canvasRef,
    exitGameModalRef,
    gameId,
    muted,
    playing,
    snapshotsModalRef,
  ]);

  const [expanded, setExpanded] = useState(false);
  return (
    <>
      <div
        className="fixed bottom-10px w-full flex justify-center"
        style={{
          visibility: expanded ? "visible" : "hidden",
        }}
      >
        <div className="bg-white/30 backdrop-blur-md py-2 pr-30px pl-3 rounded-10px relative">
          <OperationBar items={items} />
          <IconExpandDown
            className="absolute top-50% right-6px transform-translate-y--50%"
            onClick={() => {
              setExpanded(false);
            }}
          />
        </div>
      </div>

      <IconExpandDown
        className="fixed bottom-0 left-0 right-0 m-auto transform-rotate-180"
        style={{
          visibility: expanded ? "hidden" : "visible",
        }}
        onClick={() => {
          setExpanded(true);
        }}
      />
    </>
  );
}
