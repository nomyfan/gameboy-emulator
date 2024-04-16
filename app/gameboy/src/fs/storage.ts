import type { IGameManifest } from "../types";
import * as utils from "../utils";

import * as fs from ".";

export async function initCartStorage(file: File) {
  const hash = await utils.hashFile(file);
  const rootPath = `/gbos/games/cart-${hash}/`;
  if (await fs.exists(rootPath)) {
    return false;
  }

  const rootHandle = await fs.createDir(rootPath);
  await rootHandle.getDirectoryHandle("snapshots", {
    create: true,
  });

  await rootHandle
    .getFileHandle(`cart-${hash}.gb`, {
      create: true,
    })
    .then(async (handle) => {
      const writer = await handle.createWritable();
      await writer.write(file);
      await writer.close();
    });
  // TODO: cover.jpg

  const manifest: IGameManifest = {
    directory: rootPath,
    name: "Unknown", // TODO: read name from cartridge
    snapshots: [],
  };

  await rootHandle
    .getFileHandle("manifest.json", { create: true })
    .then(async (handle) => {
      const writer = await handle.createWritable();
      await writer.write(JSON.stringify(manifest, null, 2));
      await writer.close();
    });

  return true;
}

export async function deleteGame(path: string) {
  await fs.rmrf(path);
}

export async function loadAllGames() {
  const gamesDir = (await fs.resolve(
    "/gbos/games/",
  )) as FileSystemDirectoryHandle;

  const manifests: IGameManifest[] = [];
  for await (const entry of gamesDir.values()) {
    if (entry.kind === "directory") {
      const handle = entry as FileSystemDirectoryHandle;
      const manifestFile = await handle.getFileHandle("manifest.json");
      const manifest = JSON.parse(
        await (await manifestFile.getFile()).text(),
      ) as IGameManifest;
      manifests.push(manifest);
    }
  }

  return manifests;
}
