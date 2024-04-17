import { obtainMetadata } from "gb-wasm";

import type { IGameManifest } from "../types";
import * as utils from "../utils";

import * as fs from ".";

export async function installGame(file: File) {
  const hash = await utils.hashFile(file);
  const rootPath = `/gbos/games/cart-${hash}/`;
  if (await fs.exists(rootPath)) {
    return false;
  }

  const rootHandle = await fs.createDir(rootPath);
  await rootHandle.getDirectoryHandle("snapshots", {
    create: true,
  });

  await fs.createFile("rom.gb", rootHandle).then(async (handle) => {
    const writer = await handle.createWritable();
    await writer.write(file);
    await writer.close();
  });

  const metadata = await obtainMetadata(
    new Uint8ClampedArray(await file.arrayBuffer()),
    90,
  );

  await fs.createFile("cover.jpeg", rootHandle).then(async (handle) => {
    const writer = await handle.createWritable();
    await writer.write(metadata.cover);
    await writer.close();
  });

  const manifest: IGameManifest = {
    directory: rootPath,
    name: metadata.name,
    snapshots: [],
  };

  await rootHandle
    .getFileHandle("manifest.json", { create: true })
    .then(async (handle) => {
      const writer = await handle.createWritable();
      await writer.write(JSON.stringify(manifest, null, 2));
      await writer.close();
    });

  metadata.free();

  return true;
}

export async function uninstallGame(path: string) {
  await fs.rmrf(path);
}

export async function loadAllGames() {
  const gamesDir = await fs.createDir("/gbos/games/");

  const manifests: IGameManifest[] = [];
  for await (const entry of gamesDir.values()) {
    if (entry.kind === "directory") {
      const dirHandle = entry as FileSystemDirectoryHandle;
      const manifest = await dirHandle
        .getFileHandle("manifest.json")
        .then((handle) => handle.getFile())
        .then((file) => file.text())
        .then((text) => JSON.parse(text) as IGameManifest);
      manifests.push(manifest);
    }
  }

  return manifests;
}
