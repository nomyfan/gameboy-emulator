export type IGameBoyButton =
  | "UP"
  | "RIGHT"
  | "DOWN"
  | "LEFT"
  | "A"
  | "B"
  | "START"
  | "SELECT";

export type IDirectionButton = Extract<
  IGameBoyButton,
  "UP" | "DOWN" | "LEFT" | "RIGHT"
>;

export type IAbButton = Extract<IGameBoyButton, "A" | "B">;

export type IFnButton = Extract<IGameBoyButton, "START" | "SELECT">;

export interface IGameManifest {
  /**
   * Root directory for current game
   */
  directory: string;
  /**
   * Game display name
   */
  name: string;
  snapshots: Array<{
    /**
     * User can edit with a more semantic name
     * @default Formatted `datetime`
     */
    name: string;
    /**
     * Last modified datetime.
     */
    datetime: number;
    /**
     * Snapshot file name, `[snapshot-uuid].ss`
     */
    filename: string;
  }>;
}
