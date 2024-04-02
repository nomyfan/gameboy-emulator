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
