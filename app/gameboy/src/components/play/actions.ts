import { GameBoyControl, JoypadButton } from "gameboy/gameboy";
import { storage } from "gameboy/storage/indexdb";
import { actions } from "gameboy/store";
import type { IGameBoyButton } from "gameboy/types";
import * as utils from "gameboy/utils";

export const gameboy = new GameBoyControl();

export function handleButtonDown(button: IGameBoyButton) {
  gameboy.changeButton(JoypadButton[button], true);
}

export function handleButtonUp(button: IGameBoyButton) {
  gameboy.changeButton(JoypadButton[button], false);
}

export async function takeSnapshot(
  canvas: HTMLCanvasElement | null,
  gameId: string | undefined,
) {
  if (!canvas || !gameId) {
    return;
  }

  const snapshot = gameboy.takeSnapshot();
  const time = Date.now();
  const offscreenCanvas = new OffscreenCanvas(320, 288);
  offscreenCanvas
    .getContext("2d")
    ?.drawImage(
      canvas,
      0,
      0,
      canvas.width,
      canvas.height,
      0,
      0,
      offscreenCanvas.width,
      offscreenCanvas.height,
    );
  const cover = await utils.canvasToBlob(offscreenCanvas, "image/jpeg", 0.7);
  const hash = utils.hash(snapshot);
  await storage.snapshotStore.insert({
    data: snapshot,
    gameId,
    time,
    name: "Snapshot",
    cover,
    hash,
  });
}

export async function switchSnapshot(snapshot: Uint8Array) {
  await actions.openConfirmModal({
    title: "替换进度",
    content: "确定要加载该存档吗？请确保当前进度已保存。",
  });
  gameboy.restoreSnapshot(snapshot);
  gameboy.play();
}

export async function deleteSnapshot(id: number) {
  await actions.openConfirmModal({
    title: "删除",
    content: "确认要删除该存档吗？",
  });
  await storage.snapshotStore.delete(id);
}
