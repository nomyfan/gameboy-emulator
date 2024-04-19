import type { Table } from "dexie";
import { Dexie } from "dexie";
import type { IGame, IGameBoyStorage, ISnapshot } from "gameboy/model";
import type { RequiredSome } from "gameboy/types";
import * as utils from "gameboy/utils";
import { obtainMetadata } from "gb-wasm";

class DB extends Dexie {
  games!: Table<IGame, string>;
  snapshots!: Table<ISnapshot, number>;

  constructor() {
    super("gbos");
    this.version(1).stores({
      games: "&id",
      snapshots: "++id,gameId",
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

  async installGame(file: File): Promise<boolean> {
    const metadata = await file
      .arrayBuffer()
      .then((buf) => new Uint8ClampedArray(buf))
      .then((buf) => obtainMetadata(buf, 90));

    const id = await utils.hashFile(file);
    await this.gameStore.insert({
      id,
      cover: metadata.cover,
      createTime: Date.now(),
      lastPlayTime: 0,
      name: metadata.name,
      rom: file,
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
      async (tx) => {
        // Delete the game
        tx.games.delete(id);

        // Delete snapshots associated with the game
        const snapshots = await tx.snapshots.where({ gameId: id }).toArray();
        const snapshotIds = snapshots.map((s) => s.id);
        tx.snapshots.bulkDelete(snapshotIds);
      },
    );
  }
}

export const storage = new GameBoyStorage(new DB());
