import type { Table } from "dexie";
import { Dexie } from "dexie";
import { obtainMetadata } from "gb-wasm";

import type { IGame, IGameBoyStorage, ISnapshot } from "../model";
import type { RequiredSome } from "../types";
import * as utils from "../utils";

class DB extends Dexie {
  games!: Table<IGame, string>;
  snapshots!: Table<ISnapshot, number>;

  constructor() {
    super("gbos");
    this.version(1).stores({
      games: "&id",
      snapshots: "++id,game_id",
    });
  }
}

const db = new DB();

class GameStore {
  private readonly db: { games: Table<IGame, string> };

  constructor(db: { games: Table<IGame, string> }) {
    this.db = db;
  }

  async queryAll() {
    return db.games.toArray();
  }

  async queryById(id: string) {
    return db.games.get(id);
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
    return this.db.snapshots.where("game_id").equals(gameId).toArray();
  }

  async upsert(snapshot: ISnapshot) {
    return this.db.snapshots.put(snapshot);
  }

  async delete(id: number) {
    return this.db.snapshots.delete(id);
  }
}

class GameBoyStorage implements IGameBoyStorage {
  gameStore: GameStore;
  snapshotStore: SnapshotStore;

  constructor() {
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
      create_time: Date.now(),
      last_play_time: 0,
      name: metadata.name,
      rom: file,
    });
    return true;
  }

  async loadAllGames(): Promise<IGame[]> {
    const games = await this.gameStore.queryAll();
    games.sort((x, y) => {
      const xLastPlayTime = x.last_play_time ?? 0;
      const yLastPlayTime = y.last_play_time ?? 0;
      if (yLastPlayTime === xLastPlayTime) {
        return y.create_time - x.create_time;
      }
      return yLastPlayTime - xLastPlayTime;
    });

    return games;
  }

  async uninstallGame(id: string): Promise<void> {
    await this.gameStore.delete(id);
  }
}

export const storage = new GameBoyStorage();
