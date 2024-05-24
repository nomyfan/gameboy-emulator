import type { Table } from "dexie";
import { Dexie } from "dexie";
import { zip, IZipDataEntry, unzip } from "gameboy/fs/zip";
import type { IGame, IGameBoyStorage, ISnapshot } from "gameboy/model";
import type { RequiredSome } from "gameboy/types";
import * as utils from "gameboy/utils";
import { obtainMetadata } from "gb_wasm";

type IZipManifest = {
  name: string;
  id: string;
  snapshots: { name: string; time: number; hash: string }[];
};

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
            snapshot.hash = utils.hash(snapshot.data);
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

  async update(game: RequiredSome<IGame, "id">) {
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

  async queryByGameId(gameId: string) {
    return this.db.snapshots.where({ gameId }).toArray();
  }

  async insert(snapshot: Omit<ISnapshot, "id">) {
    return this.db.snapshots.add(snapshot as ISnapshot);
  }

  async insertBulk(snapshots: Omit<ISnapshot, "id">[]) {
    return this.db.snapshots.bulkAdd(snapshots as ISnapshot[]);
  }

  async delete(id: number) {
    return this.db.snapshots.delete(id);
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
    const metadata = await rom
      .arrayBuffer()
      .then((buf) => obtainMetadata(new Uint8ClampedArray(buf), 90));

    const id = await utils.hash(rom);

    await this.gameStore.insert({
      id,
      cover: metadata.cover,
      createTime: Date.now(),
      lastPlayTime: 0,
      name: metadata.name,
      rom,
    });

    metadata.free();
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

  async exportGame(id: string) {
    const game = (await this.gameStore.queryById(id))!;
    const snapshots = await this.snapshotStore.queryByGameId(id);

    const zipManifest: IZipManifest = {
      name: game.name,
      id: game.id,
      snapshots: [],
    };

    const entries: IZipDataEntry[] = [
      {
        path: game.name,
        data: game.rom,
      },
    ];

    if (game.sav) {
      entries.push({
        path: `${game.name}.sav`,
        data: game.sav,
      });
    }

    for (const snapshot of snapshots) {
      entries.push({
        path: `snapshots/${snapshot.hash}.ss`,
        data: snapshot.data,
      });
      entries.push({
        path: `snapshots/${snapshot.hash}.jpg`,
        data: snapshot.cover,
      });
      zipManifest.snapshots.push({
        name: snapshot.name,
        time: snapshot.time,
        hash: snapshot.hash,
      });
    }

    entries.push({ path: "manifest.json", data: zipManifest });

    const content = await zip(entries, {
      mimeType: "application/zip",
      level: 9,
    });
    const checksum = await utils.crc32(content);

    const pack = await zip(
      [
        { path: "data", data: content },
        { path: "checksum", data: checksum },
      ],
      { level: 0, mimeType: "application/gbpack" },
    );

    return { pack, filename: game.name };
  }

  async importGame(zipFile: File) {
    const packReader = await unzip(zipFile);
    const pack = {
      data: await packReader.getBlob("data"),
      checksum: await packReader.getText("checksum"),
    };
    if (
      !pack.data ||
      !pack.checksum ||
      pack.checksum !== (await utils.crc32(pack.data))
    ) {
      throw new Error("Broken backup");
    }

    const zipReader = await unzip(pack.data);
    const manifest =
      (await zipReader.getObject<IZipManifest>("manifest.json"))!;
    const rom = (await zipReader.getBlob(manifest.name))!;
    const metadata = await rom
      .arrayBuffer()
      .then((buf) => obtainMetadata(new Uint8ClampedArray(buf), 90));
    const sav = await zipReader.getUint8Array(`${manifest.name}.sav`);

    const snapshots: Omit<ISnapshot, "id">[] = [];
    const hashSet = await this.snapshotStore
      .queryByGameId(manifest.id)
      .then((snapshots) => new Set(snapshots.map((s) => s.hash)));

    await Promise.all(
      manifest.snapshots.map(async (snapshot) => {
        if (!hashSet.has(snapshot.hash)) {
          const ss = (await zipReader.getUint8Array(
            `snapshots/${snapshot.hash}.ss`,
          ))!;
          const cover = (await zipReader.getBlob(
            `snapshots/${snapshot.hash}.jpg`,
          ))!;
          snapshots.push({
            data: ss,
            gameId: manifest.id,
            time: snapshot.time,
            name: snapshot.name,
            cover,
            hash: snapshot.hash,
          });
        }
      }),
    );

    await this.db
      .transaction("rw", [this.db.games, this.db.snapshots], async () => {
        if (!(await this.gameStore.queryById(manifest.id))) {
          await this.gameStore.insert({
            id: manifest.id,
            cover: metadata.cover,
            createTime: Date.now(),
            lastPlayTime: 0,
            name: metadata.name,
            rom,
          });
        }
        if (sav) {
          await this.gameStore.update({ id: manifest.id, sav });
        }

        await this.snapshotStore.insertBulk(snapshots);
      })
      .finally(() => {
        metadata.free();
      });
  }
}

export const storage = new GameBoyStorage(new DB());
