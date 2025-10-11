import { OperationBar } from "gameboy/components/core/operation-bar";
import type { IExitGameModalRef } from "gameboy/components/play/exit-game-modal";
import type { ISnapshotsModalRef } from "gameboy/components/snapshots/SnapshotsModal";
import { useToast } from "gameboy/components/toast/useToast";
import { ModalCanceledError } from "gameboy/model/error";
import { storage } from "gameboy/storage/indexdb";
import {
  closePlayModal,
  openSettingsModal,
  useAppStore,
} from "gameboy/store/app";
import type { RefObject } from "react";
import { useMemo, useState } from "react";
import { useStore } from "zustand";

import { gameboy, takeSnapshot } from "./actions";

export function PlayOperationBar(props: {
  canvasRef: RefObject<HTMLCanvasElement | null>;
  snapshotsModalRef: RefObject<ISnapshotsModalRef | null>;
  exitGameModalRef: RefObject<IExitGameModalRef | null>;
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
          icon: muted ? (
            <i className="iconify ic--outline-volume-up" />
          ) : (
            <i className="iconify ic--outline-volume-off" />
          ),
          onClick: () => {
            gameboy.mute();
          },
        },
        {
          id: "take-snapshot",
          icon: <i className="iconify ic--outline-save" />,
          onClick: async () => {
            await takeSnapshot(canvasRef.current, gameId);
            addToast("已创建存档");
          },
        },
        {
          id: "snapshots",
          icon: <i className="iconify ic--baseline-manage-history" />,
          onClick: () => {
            snapshotsModalRef.current?.open();
          },
        },
        {
          id: "play-pause",
          icon: playing ? (
            <i className="iconify ic--outline-pause" />
          ) : (
            <i className="iconify ic--outline-play-arrow" />
          ),
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
          icon: <i className="iconify ic--outline-exit-to-app rotate-180" />,
          alert: true,
          onClick: async () => {
            gameboy.pause();
            exitGameModalRef.current
              ?.open()
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
                  closePlayModal("snapshot");
                } else if (action === "no_snapshot") {
                  closePlayModal("no_snapshot");
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
          icon: <i className="iconify ic--outline-settings" />,
          onClick: async () => {
            gameboy.pause();
            await openSettingsModal();
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
        className="fixed bottom-[10px] w-full flex justify-center"
        style={{
          visibility: expanded ? "visible" : "hidden",
        }}
      >
        <div className="bg-white/30 backdrop-blur-md py-2 pr-[30px] pl-3 rounded-[10px] relative">
          <OperationBar items={items} className="text-2xl" />
          <i
            className="iconify ic--outline-expand-more absolute top-1/2 right-[6px] -translate-y-1/2 text-2xl"
            onClick={() => {
              setExpanded(false);
            }}
          />
        </div>
      </div>

      <i
        className="iconify ic--outline-expand-more fixed bottom-0 left-0 right-0 m-auto rotate-180 text-2xl"
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
