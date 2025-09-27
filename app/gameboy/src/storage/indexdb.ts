import type { Table } from "dexie";
import { Dexie } from "dexie";
import type { IZipDataEntry } from "gameboy/fs/zip";
import { unzip, zip } from "gameboy/fs/zip";
import type { IGame, IGameBoyStorage, ISnapshot } from "gameboy/model";
import { crc32, flowAsync, hash } from "gameboy/utils";
import { obtainMetadata } from "gb_wasm";

type IPackManifest = {
  gameId: string;
  game: {
    /**
     * Files:
     * - game/rom
     */
    rom?: boolean;
    /**
     * Files:
     * - game/sav
     */
    sav?: boolean;
  };
  /**
   * Files:
   * - snapshots/[hash].ss
   * - snapshots/[hash].jpg: cover
   */
  snapshots: Array<{
    name: string;
    time: number;
    hash: string;
  }>;
};

async function pack(entries: IZipDataEntry[]) {
  const data = await zip(entries, {
    mimeType: "application/zip",
    level: 9,
  });
  const checksum = await crc32(data);
  return await zip(
    [
      { path: "data", data: data },
      { path: "checksum", data: checksum },
    ],
    {
      mimeType: "application/gbpack",
      level: 0,
    },
  );
}

async function unpack(pack: Blob) {
  const packReader = await unzip(pack);
  const data = await packReader.getBlob("data");
  const checksum = await packReader.getText("checksum");
  if (!data || !checksum || checksum !== (await crc32(data))) {
    throw new Error("Broken pack");
  }

  return await unzip(data);
}

class DB extends Dexie {
  games!: Table<IGame, string>;
  snapshots!: Table<ISnapshot, number>;

  constructor() {
    super("gbos");
    this.version(1).stores({
      games: "&id",
      snapshots: "++id,gameId",
    });
    this.version(2)
      .stores({
        games: "&id",
        snapshots: "++id,gameId,hash",
      })
      .upgrade(async (tx) => {
        await tx
          .table("snapshots")
          .toCollection()
          .modify((snapshot) => {
            snapshot.hash = hash(snapshot.data);
          });
      });
  }
}

class GameStore {
  private readonly db: { games: Table<IGame, string> };

  constructor(db: { games: Table<IGame, string> }) {
    this.db = db;
  }

  async queryAll() {
    return this.db.games.toArray();
  }

  async queryById(id: string) {
    return this.db.games.get(id);
  }

  async insert(game: IGame) {
    this.db.games.add(game, game.id);
  }

  async update(game: Partial<IGame> & Pick<IGame, "id">) {
    const entry = await this.db.games.get(game.id);
    if (!entry) {
      return false;
    }

    await this.db.games.update(game.id, { ...entry, ...game });

    return true;
  }

  async delete(id: string) {
    await this.db.games.delete(id);
  }
}

class SnapshotStore {
  private readonly db: { snapshots: Table<ISnapshot, number> };
  constructor(db: { snapshots: Table<ISnapshot, number> }) {
    this.db = db;
  }

  async queryById(id: number) {
    return await this.db.snapshots.get(id);
  }

  async queryByGameId(gameId: string) {
    return this.db.snapshots.where({ gameId }).toArray();
  }

  async insert(snapshot: Omit<ISnapshot, "id">) {
    return this.db.snapshots.add(snapshot as ISnapshot);
  }

  async insertBulk(snapshots: Omit<ISnapshot, "id">[]) {
    return this.db.snapshots.bulkAdd(snapshots as ISnapshot[]);
  }

  async delete(id: number | number[]) {
    if (Array.isArray(id)) {
      await this.db.snapshots.bulkDelete(id);
    } else {
      await this.db.snapshots.delete(id);
    }
  }
}

class GameBoyStorage implements IGameBoyStorage {
  db: DB;
  gameStore: GameStore;
  snapshotStore: SnapshotStore;

  constructor(db: DB) {
    this.db = db;
    this.gameStore = new GameStore(db);
    this.snapshotStore = new SnapshotStore(db);
  }

  async installGame(rom: Blob): Promise<boolean> {
    const romBuf = await rom.arrayBuffer();
    const metadata = await obtainMetadata(new Uint8ClampedArray(romBuf), 90);

    const id = await hash(rom);

    await this.gameStore.insert({
      id,
      cover: metadata.cover,
      createTime: Date.now(),
      lastPlayTime: 0,
      name: metadata.name,
      rom,
    });

    return true;
  }

  async loadAllGames(): Promise<IGame[]> {
    const games = await this.gameStore.queryAll();
    games.sort((x, y) => {
      const xLastPlayTime = x.lastPlayTime ?? 0;
      const yLastPlayTime = y.lastPlayTime ?? 0;
      if (yLastPlayTime === xLastPlayTime) {
        return y.createTime - x.createTime;
      }
      return yLastPlayTime - xLastPlayTime;
    });

    return games;
  }

  async uninstallGame(id: string): Promise<void> {
    await this.db.transaction(
      "rw",
      [this.db.games, this.db.snapshots],
      async () => {
        await this.gameStore.delete(id);
        await this.db.snapshots.where({ gameId: id }).delete();
      },
    );
  }

  async exportGame(
    gameId: string,
    options: {
      rom?: boolean;
      sav?: boolean;
      // If omitted, then export all snapshots.
      snapshots?: number[];
    },
  ) {
    const manifest: IPackManifest = { gameId, game: {}, snapshots: [] };
    const entries: IZipDataEntry[] = [];

    const game = await this.gameStore.queryById(gameId);
    if (!game) {
      throw new Error("Game not found");
    }

    if (options.rom || options.sav) {
      if (options.rom) {
        manifest.game.rom = true;
        entries.push({ path: "game/rom", data: game.rom });
      }
      if (options.sav && game.sav) {
        manifest.game.sav = true;
        entries.push({ path: "game/sav", data: game.sav });
      }
    }

    if (options.snapshots?.length !== 0) {
      const snapshotIds = options.snapshots;
      const snapshots = (await this.snapshotStore.queryByGameId(gameId)).filter(
        (s) => (snapshotIds === undefined ? true : snapshotIds.includes(s.id)),
      );
      for (const snapshot of snapshots) {
        entries.push({
          path: `snapshots/${snapshot.hash}.ss`,
          data: snapshot.data,
        });
        entries.push({
          path: `snapshots/${snapshot.hash}.jpg`,
          data: snapshot.cover,
        });
        manifest.snapshots.push({
          name: snapshot.name,
          time: snapshot.time,
          hash: snapshot.hash,
        });
      }
    }

    entries.push({ path: "manifest.json", data: manifest });

    return { pack: await pack(entries), filename: game.name };
  }

  async importPack(pack: Blob) {
    const reader = await unpack(pack);
    // biome-ignore lint/style/noNonNullAssertion: It must exist in the pack if it's exported by us.
    const manifest = (await reader.getObject<IPackManifest>("manifest.json"))!;

    const game = await this.gameStore.queryById(manifest.gameId);

    let txAction = async () => {};

    if (manifest.game.rom) {
      // biome-ignore lint/style/noNonNullAssertion: It must exist in the pack if it's exported by us.
      const rom = (await reader.getBlob("game/rom"))!;
      const romBuf = await rom.arrayBuffer();
      const metadata = await obtainMetadata(new Uint8ClampedArray(romBuf), 90);
      const sav = await reader.getUint8Array("game/sav");
      txAction = async () => {
        if (!game) {
          await this.gameStore.insert({
            id: manifest.gameId,
            cover: metadata.cover,
            createTime: Date.now(),
            lastPlayTime: 0,
            name: metadata.name,
            rom,
            sav,
          });
        } else if (sav) {
          await this.gameStore.update({ id: manifest.gameId, sav });
        }
      };
    } else if (manifest.game.sav && game) {
      const sav = await reader.getUint8Array("game/sav");
      txAction = flowAsync(txAction, async () => {
        await this.gameStore.update({ id: manifest.gameId, sav });
      });
    } else if (!game) {
      throw new Error("Game not found");
    }

    if (manifest.snapshots.length) {
      const snapshots: Omit<ISnapshot, "id">[] = [];
      const hashSet = await this.snapshotStore
        .queryByGameId(manifest.gameId)
        .then((snapshots) => new Set(snapshots.map((s) => s.hash)));
      await Promise.all(
        manifest.snapshots.map(async (snapshot) => {
          if (!hashSet.has(snapshot.hash)) {
            // biome-ignore lint/style/noNonNullAssertion: It must exist in the pack if it's exported by us.
            const ss = (await reader.getUint8Array(
              `snapshots/${snapshot.hash}.ss`,
            ))!;
            // biome-ignore lint/style/noNonNullAssertion: It must exist in the pack if it's exported by us.
            const cover = (await reader.getBlob(
              `snapshots/${snapshot.hash}.jpg`,
            ))!;
            snapshots.push({
              data: ss,
              gameId: manifest.gameId,
              time: snapshot.time,
              name: snapshot.name,
              cover,
              hash: snapshot.hash,
            });
          }
        }),
      );
      txAction = flowAsync(txAction, async () => {
        await this.snapshotStore.insertBulk(snapshots);
      });
    }

    await this.db.transaction(
      "rw",
      [this.db.games, this.db.snapshots],
      txAction,
    );
  }
}

export const storage = new GameBoyStorage(new DB());
