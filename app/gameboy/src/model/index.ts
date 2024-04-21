export interface IGame {
  id: string;
  rom: Blob;
  sav?: Uint8Array;
  cover: Blob;
  name: string;
  createTime: number;
  lastPlayTime?: number;
}

export interface ISnapshot {
  id: number;
  name: string;
  time: number;
  data: Uint8Array;
  cover: Blob;
  gameId: IGame["id"];
}

export interface IGameBoyStorage {
  installGame(file: File): Promise<boolean>;
  uninstallGame(id: string): Promise<void>;
  loadAllGames(): Promise<IGame[]>;
}
