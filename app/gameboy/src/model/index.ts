export interface IGame {
  id: string;
  rom: Blob;
  sav?: Blob;
  cover: Blob;
  name: string;
  create_time: number;
  last_play_time?: number;
}

export interface ISnapshot {
  id: number;
  name: string;
  time: number;
  data: Uint8Array;
  cover: Blob;
  game_id: IGame["id"];
}

export interface IGameBoyStorage {
  installGame(file: File): Promise<boolean>;
  uninstallGame(id: string): Promise<void>;
  loadAllGames(): Promise<IGame[]>;
}
